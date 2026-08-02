//! Core domain models for invariant analysis.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A compiled invariant expression with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    /// Unique identifier for the invariant.
    pub name: String,

    /// Human-readable description.
    pub description: Option<String>,

    /// The invariant expression in IR form.
    pub expression: Expression,

    /// Severity level: "critical", "high", "medium", "low".
    pub severity: String,

    /// Category: "core", "defi", "bridge", "governance", "account-abstraction", etc.
    pub category: String,

    /// Whether this invariant should always hold.
    pub is_always_true: bool,

    /// Layer scopes for cross-layer analysis (e.g., ["bundler", "account", "paymaster"]).
    /// If empty, applies to all layers.
    pub layers: Vec<String>,

    /// Execution phases (e.g., ["validation", "execution", "settlement"]).
    /// For AA invariants that must hold at specific phases. Empty means all phases.
    pub phases: Vec<String>,
}

/// An expression tree representing invariant conditions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Expression {
    /// Boolean literal.
    Boolean(bool),

    /// Variable reference.
    Var(String),

    /// Layer-qualified variable reference (e.g., bundler::nonce).
    LayerVar {
        /// Layer name (bundler, account, paymaster, protocol, entrypoint).
        layer: String,
        /// Variable name within the layer.
        var: String,
    },

    /// Phase-qualified variable reference (e.g., validation::account::balance).
    /// Checks state at a specific execution phase (validation, execution, settlement).
    PhaseQualifiedVar {
        /// Execution phase: "validation", "execution", or "settlement".
        phase: String,
        /// Layer name within that phase.
        layer: String,
        /// Variable name within the layer.
        var: String,
    },

    /// Phase constraint: ensures a condition holds during a specific phase.
    /// Example: ensure `account::balance >= min_required` during validation phase.
    PhaseConstraint {
        /// Phase to enforce the constraint in.
        phase: String,
        /// The constraint expression to hold in this phase.
        constraint: Box<Expression>,
    },

    /// Cross-phase relation: relates variable values across two phases.
    /// Example: `validation::account::balance >= execution::account::balance`
    /// used to track state changes across phases.
    CrossPhaseRelation {
        /// First phase.
        phase1: String,
        /// First phase expression.
        expr1: Box<Expression>,
        /// Second phase.
        phase2: String,
        /// Second phase expression.
        expr2: Box<Expression>,
        /// Relation operator.
        op: BinaryOp,
    },

    /// Integer constant.
    Int(i128),

    /// Comparison: left op right.
    BinaryOp {
        /// Left operand.
        left: Box<Expression>,
        /// Operator: ==, !=, <, >, <=, >=.
        op: BinaryOp,
        /// Right operand.
        right: Box<Expression>,
    },

    /// Arithmetic: left op right, yielding a number rather than a boolean.
    ///
    /// Conservation properties — `sum(balances) == totalSupply`,
    /// `reserve_a * reserve_b >= k` — are the whole point of an invariant
    /// language, and none of them are expressible without this.
    Arithmetic {
        /// Left operand.
        left: Box<Expression>,
        /// Operator: +, -, *, /.
        op: ArithOp,
        /// Right operand.
        right: Box<Expression>,
    },

    /// Logical operation: &&, ||.
    Logical {
        /// Left operand.
        left: Box<Expression>,
        /// Operator: And, Or.
        op: LogicalOp,
        /// Right operand.
        right: Box<Expression>,
    },

    /// Logical negation.
    Not(Box<Expression>),

    /// Function call.
    FunctionCall {
        /// Function name.
        name: String,
        /// Arguments.
        args: Vec<Expression>,
    },

    /// Tuple of expressions.
    Tuple(Vec<Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Var(v) => write!(f, "{}", v),
            Self::LayerVar { layer, var } => write!(f, "{}::{}", layer, var),
            Self::PhaseQualifiedVar { phase, layer, var } => {
                write!(f, "{}::{}::{}", phase, layer, var)
            }
            Self::PhaseConstraint { phase, constraint } => {
                write!(f, "({} @ {})", constraint, phase)
            }
            Self::CrossPhaseRelation {
                phase1,
                expr1,
                phase2,
                expr2,
                op,
            } => {
                write!(f, "({}[{}] {} {}[{}])", expr1, phase1, op, expr2, phase2)
            }
            Self::Int(i) => write!(f, "{}", i),
            Self::BinaryOp { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Self::Arithmetic { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Self::Logical { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Self::Not(e) => write!(f, "!({})", e),
            Self::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Self::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Arithmetic operators for numeric expressions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArithOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division (integer, truncating — matches EVM semantics).
    Div,
}

impl std::fmt::Display for ArithOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

/// Binary (comparison) operators for expressions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    /// Equality.
    Eq,
    /// Not equal.
    Neq,
    /// Less than.
    Lt,
    /// Greater than.
    Gt,
    /// Less than or equal.
    Lte,
    /// Greater than or equal.
    Gte,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Lte => write!(f, "<="),
            Self::Gte => write!(f, ">="),
        }
    }
}

/// Logical operators.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogicalOp {
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
}

impl std::fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}

/// A state variable in a program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVar {
    /// Variable name.
    pub name: String,

    /// Data type (chain-specific).
    pub type_name: String,

    /// Whether it's mutable.
    pub is_mutable: bool,

    /// Visibility: "public", "private", "internal", etc.
    pub visibility: Option<String>,
}

/// A function or entry point in a program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionModel {
    /// Function name.
    pub name: String,

    /// Function signature/parameters.
    pub parameters: Vec<String>,

    /// Return type.
    pub return_type: Option<String>,

    /// State variables this function mutates.
    pub mutates: BTreeSet<String>,

    /// State variables this function reads.
    pub reads: BTreeSet<String>,

    /// Whether this is an entry point.
    pub is_entry_point: bool,

    /// Whether it's pure/view (doesn't mutate state).
    pub is_pure: bool,
}

/// A complete program model extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramModel {
    /// Program/contract/module name.
    pub name: String,

    /// State variables.
    pub state_vars: BTreeMap<String, StateVar>,

    /// Functions/entry points.
    pub functions: BTreeMap<String, FunctionModel>,

    /// Functions → Mutations mapping (deterministic).
    pub mutation_graph: BTreeMap<String, BTreeSet<String>>,

    /// Chains supported: "solana", "evm", "move".
    pub chain: String,

    /// Source file path.
    pub source_path: String,
}

impl ProgramModel {
    /// Create a new program model.
    pub fn new(name: String, chain: String, source_path: String) -> Self {
        Self {
            name,
            chain,
            source_path,
            state_vars: BTreeMap::new(),
            functions: BTreeMap::new(),
            mutation_graph: BTreeMap::new(),
        }
    }

    /// Add a state variable to the model.
    pub fn add_state_var(&mut self, var: StateVar) {
        self.state_vars.insert(var.name.clone(), var);
    }

    /// Add a function to the model.
    pub fn add_function(&mut self, func: FunctionModel) {
        self.mutation_graph
            .insert(func.name.clone(), func.mutates.clone());
        self.functions.insert(func.name.clone(), func);
    }
}

/// Output from code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOutput {
    /// The generated code.
    pub code: String,

    /// Assertions injected.
    pub assertions: Vec<String>,

    /// Generated tests (if any).
    pub tests: Option<String>,

    /// Coverage percentage (0-100).
    pub coverage_percent: u8,
}

/// Report from a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    /// Number of violations found.
    pub violations: usize,

    /// Violation traces.
    pub traces: Vec<String>,

    /// Coverage percentage.
    pub coverage: f64,

    /// Deterministic seed used.
    pub seed: u64,
}
