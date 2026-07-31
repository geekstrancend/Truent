//! Chain-agnostic execution backend abstraction.
//!
//! The fuzzing engine in this crate (sequence generation, shrinking,
//! invariant checking) never touches a VM directly. It only knows how to
//! drive an [`ExecutionBackend`]. Each chain gets its own implementation:
//! `truent-dynamic-evm` provides one backed by `revm` for real Solidity
//! bytecode, and this crate's test suite provides an in-memory
//! `MockBackend` so the engine logic itself is provable without any VM.

/// A parameter type for a callable function. Deliberately covers only
/// fixed-width ("static" in Solidity ABI terms) types for the first version:
/// dynamic types (bytes/string/arrays) need length-prefixed encoding and
/// tail pointers, which is a natural follow-up once static-type coverage is
/// proven out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Uint256,
    Address,
    Bool,
    Bytes32,
}

/// Describes one callable entry point on a deployed contract, independent of
/// source language: a selector, its argument shape, and whether calling it
/// can mutate state (used to bias sequence generation and to let invariants
/// distinguish "read for oracle purposes" from "call under test").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpec {
    pub name: String,
    pub selector: [u8; 4],
    pub inputs: Vec<ParamKind>,
    pub mutates_state: bool,
    pub payable: bool,
}

impl FunctionSpec {
    pub fn new(name: &str, selector: [u8; 4], inputs: Vec<ParamKind>, mutates_state: bool) -> Self {
        Self {
            name: name.to_string(),
            selector,
            inputs,
            mutates_state,
            payable: false,
        }
    }
}

/// A single call ready to execute: fully encoded calldata plus the
/// transaction-level context (who's calling, with how much value attached).
#[derive(Debug, Clone)]
pub struct EncodedCall {
    pub function: FunctionSpec,
    pub calldata: Vec<u8>,
    pub caller: [u8; 20],
    pub value: u128,
}

/// The result of executing one [`EncodedCall`] against the backend.
#[derive(Debug, Clone)]
pub struct CallOutcome {
    pub reverted: bool,
    pub return_data: Vec<u8>,
}

/// Abstraction over "a deployed contract instance I can call into, and whose
/// state I can snapshot/restore" — implemented once per chain.
///
/// Object-safe by design (no generics, no `Self` returns) so the engine can
/// hold `&mut dyn ExecutionBackend` and invariants can be stored as
/// `Box<dyn Invariant>` without knowing the concrete backend type.
pub trait ExecutionBackend {
    /// Execute one call against the current state, mutating it.
    fn call(&mut self, call: &EncodedCall) -> CallOutcome;

    /// Snapshot current state, returning an opaque id the shrinker can
    /// later restore with [`ExecutionBackend::revert_to`]. Used so failing
    /// sequences can be replayed from a clean slate without needing a
    /// full re-deploy for every shrink candidate.
    fn snapshot(&mut self) -> u64;

    /// Restore state to a previously taken snapshot.
    fn revert_to(&mut self, snapshot: u64);

    /// The execution trace of the most recent [`ExecutionBackend::call`],
    /// for reentrancy analysis (see [`crate::trace`]). Backends that can't
    /// observe execution structure return an empty slice (the default), in
    /// which case [`crate::trace::detect_reentrancy`] simply never fires —
    /// reentrancy checking is a no-op rather than an error on such a
    /// backend. The `revm` backend overrides this via an execution
    /// inspector.
    fn last_call_trace(&self) -> &[crate::trace::TraceEvent] {
        &[]
    }
}
