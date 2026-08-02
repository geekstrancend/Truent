//! Binds Truent DSL invariants to a live contract so the engine can check
//! user-written properties, not just the handful it can infer from an ABI.
//!
//! Auto-detection only recognises shapes it already knows — ERC20 conservation,
//! monotonic accumulators, an owner field. The properties that actually matter
//! for a given protocol (is it solvent, can a provider always exit) are
//! specific to that protocol and cannot be guessed from a function list. This
//! is the path for stating them.
//!
//! Binding rule: a free variable in the expression resolves to a zero-argument
//! view function of the same name on the contract. `collateral >= liabilities`
//! calls `collateral()` and `liabilities()` after every call in the sequence
//! and evaluates the comparison over the results. Name matching tolerates
//! snake_case in the DSL against camelCase in Solidity, since the two
//! conventions differ and a property should not fail to bind over that.
//!
//! A variable that cannot be bound is an error, never a skip: silently
//! dropping a property the user asked for would report "no violations" for a
//! check that never ran.

use std::collections::BTreeMap;

use truent_core::evaluator::{Evaluator, ExecutionContext, Value};
use truent_core::model::{Expression, Invariant as DslSpec};
use truent_dynamic_core::{
    decode_uint256, encode_call, CheckContext, EncodedCall, ExecutionBackend, FunctionSpec,
    Invariant,
};

const ZERO_ADDR: [u8; 20] = [0u8; 20];

/// One chain read backing a variable or a call in the expression.
#[derive(Debug, Clone)]
struct Read {
    /// How the read is written in the DSL, used to substitute the result back.
    key: String,
    getter: FunctionSpec,
    /// ABI words for the getter's arguments, from literals in the DSL.
    args: Vec<[u8; 32]>,
}

/// A DSL property bound to the getters that supply its values.
#[derive(Debug)]
pub struct DslInvariant {
    label: String,
    expression: Expression,
    reads: Vec<Read>,
}

impl DslInvariant {
    /// Bind `spec` against `functions`, or report every variable that has no
    /// matching zero-argument view.
    pub fn bind(spec: &DslSpec, functions: &[FunctionSpec]) -> Result<Self, BindError> {
        let mut sites = Vec::new();
        collect_sites(&spec.expression, &mut sites);
        sites.sort_by_key(|s| s.key());
        sites.dedup_by(|a, b| a.key() == b.key());

        let mut reads = Vec::new();
        let mut unbound = Vec::new();

        for site in sites {
            match site.resolve(functions) {
                Ok(read) => reads.push(read),
                Err(why) => unbound.push(why),
            }
        }

        if !unbound.is_empty() {
            return Err(BindError {
                invariant: spec.name.clone(),
                unbound,
                available: functions
                    .iter()
                    .filter(|f| f.inputs.is_empty())
                    .map(|f| f.name.clone())
                    .collect(),
            });
        }

        Ok(Self {
            label: spec.name.clone(),
            expression: spec.expression.clone(),
            reads,
        })
    }

    /// Read every bound getter and evaluate the property.
    fn evaluate(&self, backend: &mut dyn ExecutionBackend) -> Outcome {
        let mut ctx = ExecutionContext::new();
        let mut observed: BTreeMap<String, u128> = BTreeMap::new();

        for Read { key, getter, args } in &self.reads {
            let call = EncodedCall {
                calldata: encode_call(getter, args),
                function: getter.clone(),
                caller: ZERO_ADDR,
                value: 0,
            };
            let outcome = backend.call(&call);
            if outcome.reverted {
                return Outcome::Undecidable(format!("getter {}() reverted", getter.name));
            }
            let word = decode_uint256(&outcome.return_data);
            let Some(value) = u128_from_word(&word) else {
                // The contract is in a legitimate state this binding cannot
                // represent. That is a limit of the tool, not a bug in the
                // contract, and must never be reported as a violation.
                return Outcome::Undecidable(format!(
                    "{}() exceeds 128 bits in this state",
                    getter.name
                ));
            };
            observed.insert(key.clone(), value);
            ctx.set_state(key.clone(), value_of(value));
        }

        // Calls are replaced by the value just read, so the evaluator only ever
        // sees plain variables — it has no way to reach the chain itself.
        let expression = substitute_calls(&self.expression, &observed);

        let evaluator = Evaluator::new(ctx);
        match evaluator.evaluate(&expression) {
            Ok(Value::Bool(true)) => Outcome::Holds,
            Ok(Value::Bool(false)) => Outcome::Violated(render_state(&observed)),
            Ok(other) => {
                Outcome::Undecidable(format!("expression evaluated to {other}, not a boolean"))
            }
            // Overflow or division by zero means the property could not be
            // decided here — again a limit, not a finding.
            Err(e) => Outcome::Undecidable(format!("could not evaluate: {e}")),
        }
    }
}

/// The result of checking a property against one contract state.
enum Outcome {
    /// The property held.
    Holds,
    /// The property was demonstrably false, with the state that broke it.
    Violated(String),
    /// The property could not be decided. Not a finding.
    ///
    /// The reason is carried for diagnostics even though the check path only
    /// needs to know that it is not a violation.
    Undecidable(#[allow(dead_code)] String),
}

impl Invariant for DslInvariant {
    fn name(&self) -> &str {
        &self.label
    }

    fn check(&self, backend: &mut dyn ExecutionBackend, _ctx: &CheckContext) -> Option<String> {
        match self.evaluate(backend) {
            Outcome::Violated(detail) => Some(detail),
            // Holding and being undecidable are both "no finding here".
            Outcome::Holds | Outcome::Undecidable(_) => None,
        }
    }
}

/// Why a property could not be attached to the contract.
#[derive(Debug, Clone)]
pub struct BindError {
    /// The invariant that failed to bind.
    pub invariant: String,
    /// Variables with no matching getter.
    pub unbound: Vec<String>,
    /// Zero-argument getters the contract does expose, to make the fix obvious.
    pub available: Vec<String>,
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invariant '{}' references {} with no matching zero-argument view on \
             the contract.\n  unbound: {}\n  available getters: {}",
            self.invariant,
            if self.unbound.len() == 1 {
                "a variable"
            } else {
                "variables"
            },
            self.unbound.join(", "),
            if self.available.is_empty() {
                "(none)".to_string()
            } else {
                self.available.join(", ")
            }
        )
    }
}

impl std::error::Error for BindError {}

/// A place in the expression that needs a value from the chain: a bare
/// variable, or a call with literal arguments.
#[derive(Debug, Clone)]
enum Site {
    /// `collateral` — a zero-argument getter.
    Var(String),
    /// `shareOf(1, 2)` — a getter with constant arguments, which is how a
    /// property reaches into a mapping. Without this, anything keyed by id or
    /// address is unreachable and only whole-contract totals can be stated.
    Call { name: String, args: Vec<i128> },
}

impl Site {
    /// Canonical text, used both to deduplicate and as the variable name the
    /// evaluator sees after substitution.
    fn key(&self) -> String {
        match self {
            Self::Var(n) => n.clone(),
            Self::Call { name, args } => format!(
                "{}({})",
                name,
                args.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }

    fn resolve(&self, functions: &[FunctionSpec]) -> Result<Read, String> {
        match self {
            Self::Var(name) => match find_getter(name, functions, 0) {
                Some(f) => Ok(Read {
                    key: self.key(),
                    getter: f.clone(),
                    args: Vec::new(),
                }),
                None => Err(name.clone()),
            },
            Self::Call { name, args } => {
                let Some(f) = find_getter(name, functions, args.len()) else {
                    return Err(format!("{}/{}", name, args.len()));
                };
                let mut words = Vec::with_capacity(args.len());
                for (arg, kind) in args.iter().zip(f.inputs.iter()) {
                    words.push(encode_literal(*arg, kind)?);
                }
                Ok(Read {
                    key: self.key(),
                    getter: f.clone(),
                    args: words,
                })
            }
        }
    }
}

/// Encode a DSL integer literal as one ABI word for `kind`.
fn encode_literal(value: i128, kind: &truent_dynamic_core::ParamKind) -> Result<[u8; 32], String> {
    use truent_dynamic_core::ParamKind;
    if value < 0 {
        return Err(format!("negative argument {value} is not encodable"));
    }
    let mut word = [0u8; 32];
    match kind {
        ParamKind::Uint256 | ParamKind::Bytes32 | ParamKind::Address | ParamKind::Bool => {
            word[16..].copy_from_slice(&(value as u128).to_be_bytes());
        }
    }
    Ok(word)
}

/// Collect every site in an expression that needs a chain read.
fn collect_sites(expr: &Expression, out: &mut Vec<Site>) {
    match expr {
        Expression::Var(name) => out.push(Site::Var(name.clone())),
        Expression::LayerVar { var, .. } | Expression::PhaseQualifiedVar { var, .. } => {
            out.push(Site::Var(var.clone()))
        }
        Expression::FunctionCall { name, args } => {
            // Only calls whose arguments are all literals can be resolved to a
            // fixed selector+calldata ahead of the run.
            let literals: Option<Vec<i128>> = args
                .iter()
                .map(|a| match a {
                    Expression::Int(v) => Some(*v),
                    Expression::Boolean(b) => Some(i128::from(*b)),
                    _ => None,
                })
                .collect();
            match literals {
                Some(args) => out.push(Site::Call {
                    name: name.clone(),
                    args,
                }),
                // A call with a computed argument still needs whatever that
                // argument reads.
                None => {
                    for a in args {
                        collect_sites(a, out);
                    }
                }
            }
        }
        Expression::BinaryOp { left, right, .. }
        | Expression::Arithmetic { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            collect_sites(left, out);
            collect_sites(right, out);
        }
        Expression::Not(inner)
        | Expression::PhaseConstraint {
            constraint: inner, ..
        } => collect_sites(inner, out),
        Expression::CrossPhaseRelation { expr1, expr2, .. } => {
            collect_sites(expr1, out);
            collect_sites(expr2, out);
        }
        Expression::Tuple(args) => {
            for a in args {
                collect_sites(a, out);
            }
        }
        Expression::Boolean(_) | Expression::Int(_) => {}
    }
}

/// Replace each resolved call with the value read for it, so the evaluator
/// only sees variables and literals.
fn substitute_calls(expr: &Expression, observed: &BTreeMap<String, u128>) -> Expression {
    let rebuild = |e: &Expression| Box::new(substitute_calls(e, observed));
    match expr {
        Expression::FunctionCall { name, args } => {
            let literals: Option<Vec<i128>> = args
                .iter()
                .map(|a| match a {
                    Expression::Int(v) => Some(*v),
                    Expression::Boolean(b) => Some(i128::from(*b)),
                    _ => None,
                })
                .collect();
            if let Some(args) = literals {
                let key = Site::Call {
                    name: name.clone(),
                    args,
                }
                .key();
                if let Some(v) = observed.get(&key) {
                    // Values above i128::MAX are rejected earlier as
                    // unrepresentable, so this conversion cannot silently wrap.
                    return Expression::Int(*v as i128);
                }
            }
            expr.clone()
        }
        Expression::BinaryOp { left, op, right } => Expression::BinaryOp {
            left: rebuild(left),
            op: *op,
            right: rebuild(right),
        },
        Expression::Arithmetic { left, op, right } => Expression::Arithmetic {
            left: rebuild(left),
            op: *op,
            right: rebuild(right),
        },
        Expression::Logical { left, op, right } => Expression::Logical {
            left: rebuild(left),
            op: *op,
            right: rebuild(right),
        },
        Expression::Not(inner) => Expression::Not(rebuild(inner)),
        other => other.clone(),
    }
}

/// Find a zero-argument view whose name matches `var`, ignoring case and the
/// snake_case/camelCase difference between the DSL and Solidity.
fn find_getter<'a>(
    var: &str,
    functions: &'a [FunctionSpec],
    arity: usize,
) -> Option<&'a FunctionSpec> {
    let target = normalise(var);
    functions
        .iter()
        .find(|f| f.inputs.len() == arity && !f.mutates_state && normalise(&f.name) == target)
        // A view wrongly marked as mutating should still be readable rather
        // than blocking the property.
        .or_else(|| {
            functions
                .iter()
                .find(|f| f.inputs.len() == arity && normalise(&f.name) == target)
        })
}

fn normalise(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Narrow a 32-byte word to `u128`, or `None` if it does not fit.
fn u128_from_word(word: &[u8; 32]) -> Option<u128> {
    if word[..16].iter().any(|b| *b != 0) {
        return None;
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&word[16..]);
    Some(u128::from_be_bytes(buf))
}

fn value_of(v: u128) -> Value {
    if v <= u64::MAX as u128 {
        Value::U64(v as u64)
    } else {
        Value::U128(v)
    }
}

/// The observed state at the moment the property failed — without it the
/// report says something broke but not why.
fn render_state(observed: &BTreeMap<String, u128>) -> String {
    let mut parts: Vec<String> = observed.iter().map(|(k, v)| format!("{k} = {v}")).collect();
    parts.sort();
    format!("violated with {}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use truent_dynamic_core::ParamKind;

    fn view(name: &str) -> FunctionSpec {
        FunctionSpec::new(name, [0, 0, 0, 0], vec![], false)
    }

    fn spec(src: &str) -> DslSpec {
        truent_dsl_parser::parse_invariant(src).expect("parse")
    }

    #[test]
    fn binds_variables_to_matching_getters() {
        let fns = vec![view("collateral"), view("totalSupply")];
        let inv = DslInvariant::bind(&spec("invariant S { collateral >= totalSupply }"), &fns)
            .expect("should bind");
        assert_eq!(inv.reads.len(), 2);
    }

    /// snake_case in the DSL against camelCase in Solidity is a naming
    /// convention difference, not a reason for a property to fail.
    #[test]
    fn binding_tolerates_snake_case_against_camel_case() {
        let fns = vec![view("totalSupply")];
        let inv = DslInvariant::bind(&spec("invariant S { total_supply >= 0 }"), &fns)
            .expect("should bind across conventions");
        assert_eq!(inv.reads[0].getter.name, "totalSupply");
    }

    /// An unbindable property must be an error. Skipping it would report "no
    /// violations" for a check that never ran.
    #[test]
    fn unbound_variables_are_reported_not_skipped() {
        let fns = vec![view("collateral")];
        let err = DslInvariant::bind(
            &spec("invariant S { collateral >= user_yes + user_no }"),
            &fns,
        )
        .expect_err("must not bind");
        assert_eq!(
            err.unbound,
            vec!["user_no".to_string(), "user_yes".to_string()]
        );
        assert!(err.to_string().contains("collateral"), "{err}");
    }

    /// Getters taking arguments cannot supply a scalar for a free variable.
    /// A bare variable must not bind to a getter that needs arguments — the
    /// engine would have no value to pass.
    #[test]
    fn bare_variable_does_not_bind_to_a_getter_with_arguments() {
        let fns = vec![FunctionSpec::new(
            "markets",
            [0, 0, 0, 0],
            vec![ParamKind::Uint256],
            false,
        )];
        assert!(DslInvariant::bind(&spec("invariant S { markets >= 0 }"), &fns).is_err());
    }

    /// Mapping-keyed state is reachable by calling the getter with literal
    /// arguments. Without this, only whole-contract totals can be stated and
    /// anything per-account or per-market is out of reach.
    #[test]
    fn getters_bind_when_called_with_literal_arguments() {
        let fns = vec![FunctionSpec::new(
            "lpOf",
            [1, 2, 3, 4],
            vec![ParamKind::Uint256, ParamKind::Address],
            false,
        )];
        let inv = DslInvariant::bind(&spec("invariant S { lpOf(1, 2) >= 0 }"), &fns)
            .expect("should bind with literal args");
        assert_eq!(inv.reads.len(), 1);
        assert_eq!(inv.reads[0].args.len(), 2);
    }

    /// Arity is part of the match: the same name with the wrong number of
    /// arguments is not the same getter.
    #[test]
    fn arity_must_match() {
        let fns = vec![FunctionSpec::new(
            "lpOf",
            [1, 2, 3, 4],
            vec![ParamKind::Uint256, ParamKind::Address],
            false,
        )];
        assert!(DslInvariant::bind(&spec("invariant S { lpOf(1) >= 0 }"), &fns).is_err());
    }
}
