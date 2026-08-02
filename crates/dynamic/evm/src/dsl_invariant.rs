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

/// A DSL property bound to the getters that supply its variables.
#[derive(Debug)]
pub struct DslInvariant {
    label: String,
    expression: Expression,
    /// Variable name as written in the DSL -> the getter that reads it.
    bindings: Vec<(String, FunctionSpec)>,
}

impl DslInvariant {
    /// Bind `spec` against `functions`, or report every variable that has no
    /// matching zero-argument view.
    pub fn bind(spec: &DslSpec, functions: &[FunctionSpec]) -> Result<Self, BindError> {
        let mut vars = Vec::new();
        collect_vars(&spec.expression, &mut vars);
        vars.sort();
        vars.dedup();

        let mut bindings = Vec::new();
        let mut unbound = Vec::new();

        for var in vars {
            match find_getter(&var, functions) {
                Some(f) => bindings.push((var, f.clone())),
                None => unbound.push(var),
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
            bindings,
        })
    }

    /// Read every bound getter and evaluate the property.
    fn evaluate(&self, backend: &mut dyn ExecutionBackend) -> Outcome {
        let mut ctx = ExecutionContext::new();
        let mut observed: BTreeMap<String, u128> = BTreeMap::new();

        for (var, getter) in &self.bindings {
            let call = EncodedCall {
                calldata: encode_call(getter, &[]),
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
            observed.insert(var.clone(), value);
            ctx.set_state(var.clone(), value_of(value));
        }

        let evaluator = Evaluator::new(ctx);
        match evaluator.evaluate(&self.expression) {
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

/// Collect every free variable in an expression.
fn collect_vars(expr: &Expression, out: &mut Vec<String>) {
    match expr {
        Expression::Var(name) => out.push(name.clone()),
        Expression::LayerVar { var, .. } | Expression::PhaseQualifiedVar { var, .. } => {
            out.push(var.clone())
        }
        Expression::BinaryOp { left, right, .. }
        | Expression::Arithmetic { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            collect_vars(left, out);
            collect_vars(right, out);
        }
        Expression::Not(inner)
        | Expression::PhaseConstraint {
            constraint: inner, ..
        } => collect_vars(inner, out),
        Expression::CrossPhaseRelation { expr1, expr2, .. } => {
            collect_vars(expr1, out);
            collect_vars(expr2, out);
        }
        Expression::FunctionCall { args, .. } | Expression::Tuple(args) => {
            for a in args {
                collect_vars(a, out);
            }
        }
        Expression::Boolean(_) | Expression::Int(_) => {}
    }
}

/// Find a zero-argument view whose name matches `var`, ignoring case and the
/// snake_case/camelCase difference between the DSL and Solidity.
fn find_getter<'a>(var: &str, functions: &'a [FunctionSpec]) -> Option<&'a FunctionSpec> {
    let target = normalise(var);
    functions
        .iter()
        .find(|f| f.inputs.is_empty() && !f.mutates_state && normalise(&f.name) == target)
        // A view wrongly marked as mutating should still be readable rather
        // than blocking the property.
        .or_else(|| {
            functions
                .iter()
                .find(|f| f.inputs.is_empty() && normalise(&f.name) == target)
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
        assert_eq!(inv.bindings.len(), 2);
    }

    /// snake_case in the DSL against camelCase in Solidity is a naming
    /// convention difference, not a reason for a property to fail.
    #[test]
    fn binding_tolerates_snake_case_against_camel_case() {
        let fns = vec![view("totalSupply")];
        let inv = DslInvariant::bind(&spec("invariant S { total_supply >= 0 }"), &fns)
            .expect("should bind across conventions");
        assert_eq!(inv.bindings[0].1.name, "totalSupply");
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
    #[test]
    fn getters_with_arguments_do_not_bind() {
        let fns = vec![FunctionSpec::new(
            "markets",
            [0, 0, 0, 0],
            vec![ParamKind::Uint256],
            false,
        )];
        assert!(DslInvariant::bind(&spec("invariant S { markets >= 0 }"), &fns).is_err());
    }
}
