//! Deterministic invariant expression evaluation engine.
//!
//! Supports both compile-time static evaluation and runtime evaluation.
//! All operations use checked arithmetic with explicit overflow handling.
//! No floating point. No randomness. No external I/O.

use crate::model::Expression;
use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A runtime value with type information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    /// Boolean value.
    Bool(bool),
    /// 64-bit unsigned integer.
    U64(u64),
    /// 128-bit unsigned integer.
    U128(u128),
    /// 64-bit signed integer.
    I64(i64),
    /// Address (hex string representation).
    Address(String),
}

impl Value {
    /// Get the type of this value.
    pub fn get_type(&self) -> Type {
        match self {
            Self::Bool(_) => Type::Bool,
            Self::U64(_) => Type::U64,
            Self::U128(_) => Type::U128,
            Self::I64(_) => Type::I64,
            Self::Address(_) => Type::Address,
        }
    }

    /// Convert to a boolean for conditional evaluation.
    pub fn to_bool(&self) -> Result<bool, EvaluationError> {
        match self {
            Self::Bool(b) => Ok(*b),
            Self::U64(n) => Ok(*n != 0),
            Self::U128(n) => Ok(*n != 0),
            Self::I64(n) => Ok(*n != 0),
            Self::Address(a) => Ok(!a.is_empty()),
        }
    }

    /// Safe integer conversion.
    #[allow(dead_code)]
    fn as_u64(&self) -> Result<u64, EvaluationError> {
        match self {
            Self::U64(n) => Ok(*n),
            Self::U128(n) => {
                if *n <= u64::MAX as u128 {
                    Ok(*n as u64)
                } else {
                    Err(EvaluationError::ConversionOverflow)
                }
            }
            Self::I64(n) => {
                if *n >= 0 {
                    Ok(*n as u64)
                } else {
                    Err(EvaluationError::ConversionOverflow)
                }
            }
            _ => Err(EvaluationError::TypeError),
        }
    }

    #[allow(dead_code)]
    fn as_i64(&self) -> Result<i64, EvaluationError> {
        match self {
            Self::I64(n) => Ok(*n),
            Self::U64(n) => {
                if *n <= i64::MAX as u64 {
                    Ok(*n as i64)
                } else {
                    Err(EvaluationError::ConversionOverflow)
                }
            }
            _ => Err(EvaluationError::TypeError),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::U64(n) => write!(f, "{}", n),
            Self::U128(n) => write!(f, "{}", n),
            Self::I64(n) => write!(f, "{}", n),
            Self::Address(a) => write!(f, "{}", a),
        }
    }
}

/// Evaluation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// Arithmetic overflow.
    Overflow,
    /// Arithmetic underflow.
    Underflow,
    /// Type error during evaluation.
    TypeError,
    /// Division by zero.
    DivisionByZero,
    /// Undefined variable.
    UndefinedVariable(String),
    /// Undefined function.
    UndefinedFunction(String),
    /// Function argument error.
    InvalidArgument(String),
    /// Conversion overflow.
    ConversionOverflow,
    /// Custom error.
    Custom(String),
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overflow => write!(f, "arithmetic overflow"),
            Self::Underflow => write!(f, "arithmetic underflow"),
            Self::TypeError => write!(f, "type error"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::UndefinedVariable(name) => write!(f, "undefined variable '{}'", name),
            Self::UndefinedFunction(name) => write!(f, "undefined function '{}'", name),
            Self::InvalidArgument(msg) => write!(f, "invalid argument: {}", msg),
            Self::ConversionOverflow => write!(f, "conversion overflow"),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// Result type for evaluation operations.
pub type EvalResult<T> = Result<T, EvaluationError>;

/// Type alias for function implementations.
pub type EvalFunction = fn(&[Value]) -> EvalResult<Value>;

/// Execution context for invariant evaluation.
pub struct ExecutionContext {
    /// Current state variable values.
    pub state_vars: BTreeMap<String, Value>,
    /// Function implementations.
    pub functions: BTreeMap<String, EvalFunction>,
}

impl ExecutionContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            state_vars: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    /// Set a state variable value.
    pub fn set_state(&mut self, name: String, value: Value) {
        self.state_vars.insert(name, value);
    }

    /// Register a built-in function.
    pub fn register_function(&mut self, name: String, func: fn(&[Value]) -> EvalResult<Value>) {
        self.functions.insert(name, func);
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Deterministic invariant expression evaluator.
pub struct Evaluator {
    context: ExecutionContext,
}

impl Evaluator {
    /// Create a new evaluator with an execution context.
    pub fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Evaluate an expression against the current context.
    pub fn evaluate(&self, expr: &Expression) -> EvalResult<Value> {
        match expr {
            Expression::Boolean(b) => Ok(Value::Bool(*b)),

            Expression::Int(val) => {
                // Determine appropriate type based on value
                if *val < 0 {
                    Ok(Value::I64(*val as i64))
                } else if *val <= u64::MAX as i128 {
                    Ok(Value::U64(*val as u64))
                } else {
                    Ok(Value::U128(*val as u128))
                }
            }

            Expression::Var(name) => self
                .context
                .state_vars
                .get(name)
                .cloned()
                .ok_or_else(|| EvaluationError::UndefinedVariable(name.clone())),

            Expression::LayerVar { layer, var } => {
                // Layer-qualified variables: look up by full qualified name
                let qualified_name = format!("{}::{}", layer, var);
                self.context
                    .state_vars
                    .get(&qualified_name)
                    .cloned()
                    .or_else(|| self.context.state_vars.get(var).cloned())
                    .ok_or(EvaluationError::UndefinedVariable(qualified_name))
            }

            Expression::PhaseQualifiedVar { phase, layer, var } => {
                // Phase-qualified variables: phase::layer::var
                // For now, evaluate as layer::var (full phase support requires AA context)
                let qualified_name = format!("{}::{}::{}", phase, layer, var);
                self.context
                    .state_vars
                    .get(&qualified_name)
                    .cloned()
                    .or_else(|| {
                        let layer_var = format!("{}::{}", layer, var);
                        self.context.state_vars.get(&layer_var).cloned()
                    })
                    .or_else(|| self.context.state_vars.get(var).cloned())
                    .ok_or(EvaluationError::UndefinedVariable(qualified_name))
            }

            Expression::PhaseConstraint {
                phase: _,
                constraint,
            } => {
                // Evaluate the constraint expression
                // The phase is metadata for analysis; actual phase checking requires AA context
                self.evaluate(constraint)
            }

            Expression::CrossPhaseRelation {
                phase1: _,
                expr1,
                phase2: _,
                expr2,
                op,
            } => {
                // Evaluate cross-phase relation: expr1 op expr2
                // Phase context requires AA context for snapshot lookup
                let left_val = self.evaluate(expr1)?;
                let right_val = self.evaluate(expr2)?;
                self.eval_binary_op(&left_val, op, &right_val)
            }

            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.eval_binary_op(&left_val, op, &right_val)
            }

            Expression::Arithmetic { left, op, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                Self::eval_arith_op(&left_val, op, &right_val)
            }

            Expression::Logical { left, op, right } => {
                use crate::model::LogicalOp;

                let left_val = self.evaluate(left)?.to_bool()?;

                // Short-circuit evaluation
                match op {
                    LogicalOp::And => {
                        if !left_val {
                            return Ok(Value::Bool(false));
                        }
                        let right_val = self.evaluate(right)?.to_bool()?;
                        Ok(Value::Bool(right_val))
                    }
                    LogicalOp::Or => {
                        if left_val {
                            return Ok(Value::Bool(true));
                        }
                        let right_val = self.evaluate(right)?.to_bool()?;
                        Ok(Value::Bool(right_val))
                    }
                }
            }

            Expression::Not(expr) => {
                let val = self.evaluate(expr)?.to_bool()?;
                Ok(Value::Bool(!val))
            }

            Expression::FunctionCall { name, args } => {
                let func = self
                    .context
                    .functions
                    .get(name)
                    .ok_or_else(|| EvaluationError::UndefinedFunction(name.clone()))?;

                let arg_vals: EvalResult<Vec<Value>> =
                    args.iter().map(|arg| self.evaluate(arg)).collect();

                func(&arg_vals?)
            }

            Expression::Tuple(exprs) => {
                // For now, evaluate first expression in tuple
                if exprs.is_empty() {
                    Ok(Value::Bool(true))
                } else {
                    self.evaluate(&exprs[0])
                }
            }
        }
    }

    /// Evaluate a binary operation with checked arithmetic.
    /// Evaluate `+ - * /` over numeric values.
    ///
    /// Every operation is checked. A conservation invariant like
    /// `reserve_a * reserve_b >= k` multiplies two token balances, which
    /// overflows `u64` at realistic magnitudes — and a wrapped product would
    /// silently satisfy the comparison, reporting a violated invariant as
    /// holding. Unsigned operands are widened to `u128` and the result is
    /// narrowed back only when it fits.
    fn eval_arith_op(left: &Value, op: &crate::model::ArithOp, right: &Value) -> EvalResult<Value> {
        use crate::model::ArithOp;

        // Signed operands evaluate in i128; everything else in u128.
        let signed = matches!(left, Value::I64(_)) || matches!(right, Value::I64(_));

        if signed {
            let to_i = |v: &Value| -> EvalResult<i128> {
                match v {
                    Value::I64(x) => Ok(*x as i128),
                    Value::U64(x) => Ok(*x as i128),
                    Value::U128(x) => {
                        i128::try_from(*x).map_err(|_| EvaluationError::ConversionOverflow)
                    }
                    _ => Err(EvaluationError::TypeError),
                }
            };
            let (l, r) = (to_i(left)?, to_i(right)?);
            let out = match op {
                ArithOp::Add => l.checked_add(r).ok_or(EvaluationError::Overflow)?,
                ArithOp::Sub => l.checked_sub(r).ok_or(EvaluationError::Underflow)?,
                ArithOp::Mul => l.checked_mul(r).ok_or(EvaluationError::Overflow)?,
                ArithOp::Div => {
                    if r == 0 {
                        return Err(EvaluationError::DivisionByZero);
                    }
                    l.checked_div(r).ok_or(EvaluationError::Overflow)?
                }
            };
            let narrowed = i64::try_from(out).map_err(|_| EvaluationError::ConversionOverflow)?;
            return Ok(Value::I64(narrowed));
        }

        let to_u = |v: &Value| -> EvalResult<u128> {
            match v {
                Value::U64(x) => Ok(*x as u128),
                Value::U128(x) => Ok(*x),
                _ => Err(EvaluationError::TypeError),
            }
        };
        let (l, r) = (to_u(left)?, to_u(right)?);
        let out = match op {
            ArithOp::Add => l.checked_add(r).ok_or(EvaluationError::Overflow)?,
            // Unsigned: a negative result is an underflow, not a wrap. EVM
            // arithmetic reverts here, so surfacing it is the honest behaviour.
            ArithOp::Sub => l.checked_sub(r).ok_or(EvaluationError::Underflow)?,
            ArithOp::Mul => l.checked_mul(r).ok_or(EvaluationError::Overflow)?,
            ArithOp::Div => {
                if r == 0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                l / r
            }
        };

        if out <= u64::MAX as u128 {
            Ok(Value::U64(out as u64))
        } else {
            Ok(Value::U128(out))
        }
    }

    fn eval_binary_op(
        &self,
        left: &Value,
        op: &crate::model::BinaryOp,
        right: &Value,
    ) -> EvalResult<Value> {
        use crate::model::BinaryOp;

        // Numeric operands are compared by value, not by representation.
        // Previously each arm required both sides to be the *same* variant, so
        // `U64(5) < U128(6)` was a TypeError and `U64(5) == U128(5)` was false.
        // Arithmetic makes mixed widths routine — `a * b` widens to U128 while
        // the bound it is compared against is often U64 — so a width mismatch
        // must not decide the result.
        if let (Some(l), Some(r)) = (Self::as_i256(left), Self::as_i256(right)) {
            return Ok(Value::Bool(match op {
                BinaryOp::Eq => l == r,
                BinaryOp::Neq => l != r,
                BinaryOp::Lt => l < r,
                BinaryOp::Gt => l > r,
                BinaryOp::Lte => l <= r,
                BinaryOp::Gte => l >= r,
            }));
        }

        match op {
            // Non-numeric (Bool, Address): equality still well-defined.
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::Neq => Ok(Value::Bool(left != right)),
            // Ordering is not.
            _ => Err(EvaluationError::TypeError),
        }
    }

    /// Widen any numeric value to a common signed type for comparison.
    ///
    /// `i128` cannot hold all of `u128`, so unsigned values above `i128::MAX`
    /// would be lost. Returning `None` for those keeps them out of the fast
    /// path rather than silently comparing a wrong number; in practice token
    /// balances never reach that magnitude.
    fn as_i256(v: &Value) -> Option<i128> {
        match v {
            Value::U64(x) => Some(*x as i128),
            Value::I64(x) => Some(*x as i128),
            Value::U128(x) => i128::try_from(*x).ok(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_detection() {
        assert_eq!(Value::Bool(true).get_type(), Type::Bool);
        assert_eq!(Value::U64(42).get_type(), Type::U64);
        assert_eq!(Value::I64(-42).get_type(), Type::I64);
    }

    #[test]
    fn test_simple_evaluation() {
        let ctx = ExecutionContext::new();
        let evaluator = Evaluator::new(ctx);

        let expr = Expression::Boolean(true);
        let result = evaluator.evaluate(&expr);
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_state_variable_evaluation() {
        let mut ctx = ExecutionContext::new();
        ctx.set_state("balance".to_string(), Value::U64(100));

        let evaluator = Evaluator::new(ctx);
        let expr = Expression::Var("balance".to_string());

        let result = evaluator.evaluate(&expr);
        assert_eq!(result, Ok(Value::U64(100)));
    }

    #[test]
    fn test_comparison_evaluation() {
        let ctx = ExecutionContext::new();
        let evaluator = Evaluator::new(ctx);

        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Int(10)),
            op: crate::model::BinaryOp::Lt,
            right: Box::new(Expression::Int(20)),
        };

        let result = evaluator.evaluate(&expr);
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_short_circuit() {
        let ctx = ExecutionContext::new();
        let evaluator = Evaluator::new(ctx);

        // false && (undefined_var) should not evaluate the right side
        let expr = Expression::Logical {
            left: Box::new(Expression::Boolean(false)),
            op: crate::model::LogicalOp::And,
            right: Box::new(Expression::Var("undefined".to_string())),
        };

        let result = evaluator.evaluate(&expr);
        assert_eq!(result, Ok(Value::Bool(false)));
    }
}
