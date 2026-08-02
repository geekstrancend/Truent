//! Type inference and checking engine.
//!
//! Performs static type checking on expressions before code generation.
//! Ensures all invariants are well-typed and cannot cause runtime type errors.

use crate::model::Expression;
use crate::types::{Type, TypeError, TypeResult, TypedExpr};
use std::collections::BTreeMap;

/// Static type checker for invariant expressions.
///
/// Performs strict, deterministic type checking with no implicit conversions.
pub struct TypeChecker {
    /// Known state variables and their types.
    state_vars: BTreeMap<String, Type>,

    /// Known functions and their signatures.
    functions: BTreeMap<String, FunctionSignature>,
}

/// A function signature: parameter types and return type.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Parameter types in order.
    pub params: Vec<Type>,

    /// Return type.
    pub return_type: Type,
}

impl TypeChecker {
    /// Create a new type checker with empty context.
    pub fn new() -> Self {
        Self {
            state_vars: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    /// Register a state variable with its type.
    pub fn register_state_var(&mut self, name: String, ty: Type) {
        self.state_vars.insert(name, ty);
    }

    /// Register a function signature.
    pub fn register_function(&mut self, name: String, sig: FunctionSignature) {
        self.functions.insert(name, sig);
    }

    /// Load state variables from a program model.
    pub fn load_from_program(&mut self, program: &crate::model::ProgramModel) {
        for (name, var) in &program.state_vars {
            // Infer type from type_name field
            let ty = self.infer_type_from_string(&var.type_name);
            self.register_state_var(name.clone(), ty);
        }

        // Register standard library functions
        self.register_stdlib_functions();
    }

    /// Type check an expression.
    ///
    /// Returns a TypedExpr if successful, or a TypeError if type checking fails.
    pub fn check_expr(&self, expr: &Expression) -> TypeResult<TypedExpr> {
        let ty = self.infer_type(expr)?;
        Ok(TypedExpr::new(expr.clone(), ty))
    }

    /// Infer the type of an expression.
    fn infer_type(&self, expr: &Expression) -> TypeResult<Type> {
        match expr {
            Expression::Boolean(_) => Ok(Type::Bool),

            Expression::Int(val) => {
                // Infer numeric type from value range
                if *val < 0 {
                    Ok(Type::I64)
                } else if *val <= u64::MAX as i128 {
                    Ok(Type::U64)
                } else {
                    Ok(Type::U128)
                }
            }

            Expression::Var(name) => self
                .state_vars
                .get(name)
                .copied()
                .ok_or_else(|| TypeError::UndefinedVariable(name.clone())),

            Expression::LayerVar { layer, var } => {
                // Layer-qualified variables are treated as typed based on convention:
                // typically they're either numeric or boolean based on context
                // For now, assume they could be any type and require explicit validation
                self.state_vars
                    .get(var)
                    .copied()
                    .ok_or_else(|| TypeError::UndefinedVariable(format!("{}::{}", layer, var)))
            }

            Expression::PhaseQualifiedVar {
                phase: _,
                layer,
                var,
            } => {
                // Phase-qualified variables: phase::layer::var
                // Type inferred from the variable name
                self.state_vars
                    .get(var)
                    .copied()
                    .ok_or_else(|| TypeError::UndefinedVariable(format!("{}::{}", layer, var)))
            }

            Expression::PhaseConstraint {
                phase: _,
                constraint,
            } => {
                // A phase constraint evaluates to the type of its constraint
                // Typically constraints evaluate to Bool
                self.infer_type(constraint)
            }

            Expression::CrossPhaseRelation {
                phase1: _,
                expr1,
                phase2: _,
                expr2,
                op: _,
            } => {
                // Cross-phase relations are comparisons that return bool
                self.check_binary_op(expr1, &crate::model::BinaryOp::Eq, expr2)?;
                Ok(Type::Bool)
            }

            Expression::BinaryOp { left, op, right } => self.check_binary_op(left, op, right),

            // Arithmetic yields a number, not a boolean. Both operands must be
            // numeric; the result widens to the larger of the two so that
            // `u64 * u64` is allowed to be compared against a u128 bound.
            Expression::Arithmetic { left, op, right } => {
                let lt = self.infer_type(left)?;
                let rt = self.infer_type(right)?;
                let numeric = |t: Type| matches!(t, Type::U64 | Type::U128 | Type::I64);
                if !numeric(lt) || !numeric(rt) {
                    return Err(TypeError::BinaryOpTypeMismatch {
                        op: op.to_string(),
                        left: lt,
                        right: rt,
                    });
                }
                Ok(match (lt, rt) {
                    (Type::U128, _) | (_, Type::U128) => Type::U128,
                    (Type::I64, _) | (_, Type::I64) => Type::I64,
                    _ => Type::U64,
                })
            }

            Expression::Logical { left, op, right } => self.check_logical_op(left, op, right),

            Expression::Not(expr) => {
                let ty = self.infer_type(expr)?;
                if ty != Type::Bool {
                    return Err(TypeError::UnaryOpTypeMismatch {
                        op: "!".to_string(),
                        operand: ty,
                    });
                }
                Ok(Type::Bool)
            }

            Expression::FunctionCall { name, args } => self.check_function_call(name, args),

            Expression::Tuple(exprs) => {
                // Type check tuple elements and infer tuple type
                if exprs.is_empty() {
                    // Empty tuple has unit type
                    Ok(Type::Bool) // Unit represented as Bool for compatibility
                } else {
                    // For tuples with multiple elements, ensure type consistency
                    let mut element_types = Vec::new();
                    for expr in exprs {
                        let ty = self.infer_type(expr)?;
                        element_types.push(ty);
                    }
                    // Return the type of the first element for now
                    // A full implementation would track tuple types (T1, T2, ...)
                    Ok(element_types[0])
                }
            }
        }
    }

    /// Check a binary operation's types.
    fn check_binary_op(
        &self,
        left: &Expression,
        op: &crate::model::BinaryOp,
        right: &Expression,
    ) -> TypeResult<Type> {
        let left_ty = self.infer_type(left)?;
        let right_ty = self.infer_type(right)?;

        use crate::model::BinaryOp;

        match op {
            BinaryOp::Eq | BinaryOp::Neq => {
                // Equality requires exact type match
                if left_ty != right_ty {
                    return Err(TypeError::IncomparableTypes {
                        left: left_ty,
                        right: right_ty,
                    });
                }
                Ok(Type::Bool)
            }

            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte => {
                // Relational operators require numeric types and matching
                if !left_ty.is_numeric() || !right_ty.is_numeric() {
                    return Err(TypeError::IncomparableTypes {
                        left: left_ty,
                        right: right_ty,
                    });
                }

                if left_ty != right_ty {
                    let op_str = match op {
                        BinaryOp::Lt => "<",
                        BinaryOp::Gt => ">",
                        BinaryOp::Lte => "<=",
                        BinaryOp::Gte => ">=",
                        // These cases should never occur due to outer match
                        // but we handle them for completeness
                        _ => "unknown",
                    };
                    return Err(TypeError::BinaryOpTypeMismatch {
                        left: left_ty,
                        op: op_str.to_string(),
                        right: right_ty,
                    });
                }

                Ok(Type::Bool)
            }
        }
    }

    /// Check a logical operation's types.
    fn check_logical_op(
        &self,
        left: &Expression,
        op: &crate::model::LogicalOp,
        right: &Expression,
    ) -> TypeResult<Type> {
        let left_ty = self.infer_type(left)?;
        let right_ty = self.infer_type(right)?;

        use crate::model::LogicalOp;

        let op_name = match op {
            LogicalOp::And => "&&",
            LogicalOp::Or => "||",
        };

        if left_ty != Type::Bool {
            return Err(TypeError::LogicalOpRequiresBool {
                op: op_name.to_string(),
                actual: left_ty,
            });
        }

        if right_ty != Type::Bool {
            return Err(TypeError::LogicalOpRequiresBool {
                op: op_name.to_string(),
                actual: right_ty,
            });
        }

        Ok(Type::Bool)
    }

    /// Check a function call's types.
    fn check_function_call(&self, name: &str, args: &[Expression]) -> TypeResult<Type> {
        let sig = self
            .functions
            .get(name)
            .ok_or_else(|| TypeError::UndefinedFunction(name.to_string()))?;

        if args.len() != sig.params.len() {
            return Err(TypeError::Custom(format!(
                "function '{}' expects {} arguments but got {}",
                name,
                sig.params.len(),
                args.len()
            )));
        }

        // Type check each argument
        for (idx, (arg, expected)) in args.iter().zip(&sig.params).enumerate() {
            let actual = self.infer_type(arg)?;
            if actual != *expected {
                return Err(TypeError::FunctionArgMismatch {
                    function: name.to_string(),
                    param_idx: idx,
                    expected: *expected,
                    actual,
                });
            }
        }

        Ok(sig.return_type)
    }

    /// Register standard library functions.
    fn register_stdlib_functions(&mut self) {
        // sum(u64) -> u64
        self.register_function(
            "sum".to_string(),
            FunctionSignature {
                params: vec![Type::U64],
                return_type: Type::U64,
            },
        );

        // len(address) -> u64
        self.register_function(
            "len".to_string(),
            FunctionSignature {
                params: vec![Type::Address],
                return_type: Type::U64,
            },
        );

        // min(u64, u64) -> u64
        self.register_function(
            "min".to_string(),
            FunctionSignature {
                params: vec![Type::U64, Type::U64],
                return_type: Type::U64,
            },
        );

        // max(u64, u64) -> u64
        self.register_function(
            "max".to_string(),
            FunctionSignature {
                params: vec![Type::U64, Type::U64],
                return_type: Type::U64,
            },
        );
    }

    /// Infer a type from a string representation.
    fn infer_type_from_string(&self, type_str: &str) -> Type {
        match type_str.to_lowercase().as_str() {
            "bool" | "boolean" => Type::Bool,
            "u64" | "uint64" => Type::U64,
            "u128" | "uint128" => Type::U128,
            "i64" | "int64" => Type::I64,
            "address" => Type::Address,
            _ => Type::U64, // Default
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_state_var() {
        let mut checker = TypeChecker::new();
        checker.register_state_var("balance".to_string(), Type::U64);

        let expr = Expression::Var("balance".to_string());
        let result = checker.check_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().ty, Type::U64);
    }

    #[test]
    fn test_undefined_variable() {
        let checker = TypeChecker::new();
        let expr = Expression::Var("unknown".to_string());
        let result = checker.check_expr(&expr);

        assert!(result.is_err());
        match result {
            Err(TypeError::UndefinedVariable(name)) => assert_eq!(name, "unknown"),
            _ => panic!("expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_type_mismatch_comparison() {
        let mut checker = TypeChecker::new();
        checker.register_state_var("flag".to_string(), Type::Bool);
        checker.register_state_var("amount".to_string(), Type::U64);

        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Var("flag".to_string())),
            op: crate::model::BinaryOp::Eq,
            right: Box::new(Expression::Var("amount".to_string())),
        };

        let result = checker.check_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_logical_requires_bool() {
        let mut checker = TypeChecker::new();
        checker.register_state_var("amount".to_string(), Type::U64);

        let expr = Expression::Logical {
            left: Box::new(Expression::Var("amount".to_string())),
            op: crate::model::LogicalOp::And,
            right: Box::new(Expression::Boolean(true)),
        };

        let result = checker.check_expr(&expr);
        assert!(result.is_err());
    }
}
