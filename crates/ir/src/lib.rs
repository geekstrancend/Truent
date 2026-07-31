#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Intermediate Representation: Chain-agnostic program model and invariant IR.
//!
//! This module re-exports core types and extends them with additional
//! IR-specific utilities while remaining chain-agnostic.

pub use truent_core::model::{
    BinaryOp, Expression, FunctionModel, GenerationOutput, Invariant, LogicalOp, ProgramModel,
    SimulationReport, StateVar,
};
pub use truent_core::{InvarError, Result};

pub mod analyzer_result;
pub mod ast;
pub mod rules;
pub mod semantic;

pub use analyzer_result::AnalysisContext;
pub use ast::DependencyGraph;
pub use semantic::{
    AuthCheckKind, AuthorizationCheck, MutationKind, PrivilegedMutation, SemanticModel,
};
