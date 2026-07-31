//! A `revm` execution inspector that records the call/storage structure of
//! a transaction as a `Vec<TraceEvent>`, which
//! [`truent_dynamic_core::detect_reentrancy`] then analyzes.
//!
//! This is the piece that makes reentrancy detection actually fire on real
//! compiled contracts: the chain-agnostic engine proved the *analysis*
//! offline against synthetic traces, and this produces the *real* traces
//! from live EVM execution.
//!
//! UNVERIFIED OFFLINE: like `backend.rs`, this is written against the
//! `revm = 14` inspector API in an environment that can't fetch `revm`, so
//! exact method/field names (`CallInputs::target_address`,
//! `Interpreter::current_opcode`, `InstructionResult` variants) are
//! validated by CI, not locally. The trace-building logic is deliberately
//! tiny and mechanical so that if a name is off, the fix is a one-liner.

use revm::interpreter::{opcode, CallInputs, CallOutcome, InstructionResult, Interpreter};
use revm::{Database, EvmContext, Inspector};
use truent_dynamic_core::TraceEvent;

/// Accumulates [`TraceEvent`]s over the course of one transaction. A fresh
/// one is used per top-level call so `events` holds exactly that call's
/// trace.
#[derive(Debug, Default, Clone)]
pub struct ReentrancyInspector {
    pub events: Vec<TraceEvent>,
}

impl ReentrancyInspector {
    pub fn take_events(&mut self) -> Vec<TraceEvent> {
        std::mem::take(&mut self.events)
    }
}

/// A call frame ends "successfully" (did not revert/halt) only for the
/// normal-return instruction results. Matching the two stable success
/// variants explicitly — rather than depending on an `is_ok()`/`is_revert()`
/// helper whose presence varies across revm versions — keeps this robust:
/// anything that isn't a clean return counts as reverted, which is the
/// conservative choice for reentrancy (a re-entry that halts didn't
/// successfully drain anything).
fn frame_reverted(result: InstructionResult) -> bool {
    !matches!(result, InstructionResult::Return | InstructionResult::Stop)
}

impl<DB: Database> Inspector<DB> for ReentrancyInspector {
    fn call(
        &mut self,
        _context: &mut EvmContext<DB>,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        self.events.push(TraceEvent::CallBegin {
            address: inputs.target_address.into_array(),
        });
        None
    }

    fn call_end(
        &mut self,
        _context: &mut EvmContext<DB>,
        _inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        self.events.push(TraceEvent::CallEnd {
            reverted: frame_reverted(outcome.result.result),
        });
        outcome
    }

    fn step(&mut self, interp: &mut Interpreter, _context: &mut EvmContext<DB>) {
        // A single u8 compare per executed opcode: cheap enough for the
        // modest run counts a fuzzing session drives, and the only way to
        // attribute storage writes to the executing contract without
        // diffing full state.
        if interp.current_opcode() == opcode::SSTORE {
            self.events.push(TraceEvent::StorageWrite {
                address: interp.contract.target_address.into_array(),
            });
        }
    }
}
