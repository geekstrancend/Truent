//! Formal type system for Truent DSL.
//!
//! This module defines a strictly typed system for invariant expressions.
//! No implicit conversions. All type errors are explicit and actionable.

use crate::model::Expression;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A formal type in the Truent type system.
///
/// Supports only deterministic, provable types. No floating point, no null.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Type {
    /// Boolean type.
    Bool,
    /// 64-bit unsigned integer.
    U64,
    /// 128-bit unsigned integer.
    U128,
    /// 64-bit signed integer.
    I64,
    /// Address type (chain-specific representation).
    Address,
}

impl Type {
    /// Check if this type is numeric.
    pub fn is_numeric(self) -> bool {
        matches!(self, Self::U64 | Self::U128 | Self::I64)
    }

    /// Check if this type is a primitive.
    pub fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::Bool | Self::U64 | Self::U128 | Self::I64 | Self::Address
        )
    }

    /// Get a human-readable name for this type.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::I64 => "i64",
            Self::Address => "address",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A value with its static type information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedValue {
    /// The static type.
    pub ty: Type,
    /// Serialized value (for inspection/logging).
    pub value: String,
}

impl TypedValue {
    /// Create a new typed value.
    pub fn new(ty: Type, value: String) -> Self {
        Self { ty, value }
    }
}

/// A typed expression after type checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedExpr {
    /// The expression.
    pub expr: Expression,
    /// The verified type.
    pub ty: Type,
}

impl TypedExpr {
    /// Create a new typed expression.
    pub fn new(expr: Expression, ty: Type) -> Self {
        Self { expr, ty }
    }
}

/// Type checking errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    /// Unknown identifier.
    UndefinedVariable(String),
    /// Unknown function.
    UndefinedFunction(String),
    /// Type mismatch in binary operation.
    BinaryOpTypeMismatch {
        /// Left operand type.
        left: Type,
        /// Operator.
        op: String,
        /// Right operand type.
        right: Type,
    },
    /// Type mismatch in unary operation.
    UnaryOpTypeMismatch {
        /// Operator.
        op: String,
        /// Operand type.
        operand: Type,
    },
    /// Function argument type mismatch.
    FunctionArgMismatch {
        /// Function name.
        function: String,
        /// Parameter index.
        param_idx: usize,
        /// Expected type.
        expected: Type,
        /// Actual type.
        actual: Type,
    },
    /// Logical operator requires boolean operand.
    LogicalOpRequiresBool {
        /// Operator.
        op: String,
        /// Actual type.
        actual: Type,
    },
    /// Comparison not allowed between types.
    IncomparableTypes {
        /// Left type.
        left: Type,
        /// Right type.
        right: Type,
    },
    /// Custom error message.
    Custom(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedVariable(name) => {
                write!(f, "undefined variable '{}'", name)
            }
            Self::UndefinedFunction(name) => {
                write!(f, "undefined function '{}'", name)
            }
            Self::BinaryOpTypeMismatch { left, op, right } => {
                write!(
                    f,
                    "type mismatch in binary operation: {} {} {} is invalid",
                    left, op, right
                )
            }
            Self::UnaryOpTypeMismatch { op, operand } => {
                write!(
                    f,
                    "type mismatch in unary operation: {}({}) is invalid",
                    op, operand
                )
            }
            Self::FunctionArgMismatch {
                function,
                param_idx,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "function '{}' parameter {} expects {} but got {}",
                    function, param_idx, expected, actual
                )
            }
            Self::LogicalOpRequiresBool { op, actual } => {
                write!(
                    f,
                    "logical operator '{}' requires bool operand, got {}",
                    op, actual
                )
            }
            Self::IncomparableTypes { left, right } => {
                write!(f, "cannot compare {} and {}", left, right)
            }
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// Result type for type checking operations.
pub type TypeResult<T> = Result<T, TypeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_names() {
        assert_eq!(Type::Bool.name(), "bool");
        assert_eq!(Type::U64.name(), "u64");
        assert_eq!(Type::Address.name(), "address");
    }

    #[test]
    fn test_numeric_types() {
        assert!(Type::U64.is_numeric());
        assert!(Type::U128.is_numeric());
        assert!(Type::I64.is_numeric());
        assert!(!Type::Bool.is_numeric());
        assert!(!Type::Address.is_numeric());
    }

    #[test]
    fn test_type_error_display() {
        let err = TypeError::UndefinedVariable("x".to_string());
        assert_eq!(err.to_string(), "undefined variable 'x'");

        let err = TypeError::BinaryOpTypeMismatch {
            left: Type::U64,
            op: "+".to_string(),
            right: Type::Bool,
        };
        assert!(err.to_string().contains("type mismatch"));
    }
}
