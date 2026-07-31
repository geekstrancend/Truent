#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Chain-agnostic dynamic/coverage-guided invariant fuzzing engine.
//!
//! This crate contains no VM: it only knows how to generate call
//! sequences, drive an [`ExecutionBackend`], check [`Invariant`]s after
//! every call, and shrink a failing sequence to a minimal reproduction.
//! Chain-specific crates (starting with `truent-dynamic-evm`, backed by
//! `revm`) provide the [`ExecutionBackend`] implementation; this crate's
//! own test suite proves the engine logic correct against an in-memory
//! mock so it never depends on a real VM being available to be verified.

pub mod abi_encode;
pub mod backend;
pub mod invariant;
pub mod poc;
pub mod sequence;
pub mod shrink;
pub mod trace;
pub mod u256;

#[cfg(test)]
mod testing;

pub use backend::{CallOutcome, EncodedCall, ExecutionBackend, FunctionSpec, ParamKind};
pub use invariant::{
    AccessControlInvariant, CheckContext, ConservationInvariant, Invariant, MonotonicInvariant,
    ReentrancyInvariant,
};
pub use poc::format_poc;
pub use sequence::{generate_random_sequence, run_sequence, CallSequence, Violation};
pub use shrink::shrink;
pub use trace::{detect_reentrancy, ReentrancyReport, TraceEvent};

use rand::rngs::SmallRng;
use rand::SeedableRng;

/// Search parameters for a fuzzing run. `seed` makes runs reproducible —
/// the same seed against the same contract always generates the same
/// sequences, so a violation found once can be found again for debugging.
#[derive(Debug, Clone)]
pub struct FuzzConfig {
    pub seed: u64,
    pub max_runs: usize,
    pub sequence_depth: usize,
    pub actors: Vec<[u8; 20]>,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            max_runs: 500,
            sequence_depth: 12,
            actors: Vec::new(),
        }
    }
}

/// Run the search loop: generate random call sequences, execute each
/// against a fresh backend instance, check invariants after every call,
/// and on the first violation, shrink it to a minimal reproduction before
/// returning.
///
/// `fresh_backend` is called once per attempt (and many more times during
/// shrinking) — it must return a backend already in its genesis/deployed
/// state, ready to receive calls. How "fresh" is produced (redeploy vs. a
/// cheap in-memory state clone) is entirely the backend's concern.
pub fn fuzz(
    fresh_backend: impl Fn() -> Box<dyn ExecutionBackend>,
    functions: &[FunctionSpec],
    invariants: Vec<Box<dyn Invariant>>,
    config: FuzzConfig,
) -> Option<Violation> {
    let mut rng = SmallRng::seed_from_u64(config.seed);

    for _ in 0..config.max_runs {
        let seq =
            generate_random_sequence(&mut rng, functions, &config.actors, config.sequence_depth);
        if seq.is_empty() {
            continue;
        }

        let mut backend = fresh_backend();
        let Some(violation) = run_sequence(backend.as_mut(), &seq, &invariants) else {
            continue;
        };

        let shrunk = shrink(&fresh_backend, &violation.sequence, &invariants);
        let mut replay_backend = fresh_backend();
        // Re-run the shrunk sequence to get a Violation object whose
        // `sequence` field matches what shrink actually produced — `shrink`
        // itself only returns the CallSequence, not a fresh Violation.
        return Some(
            run_sequence(replay_backend.as_mut(), &shrunk, &invariants).unwrap_or(violation),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{self, MockBackend};

    #[test]
    fn catches_monotonic_violation_and_shrinks_it() {
        let functions = testing::counter_functions();
        let value_fn = functions
            .iter()
            .find(|f| f.selector == testing::VALUE)
            .unwrap()
            .clone();
        let invariants: Vec<Box<dyn Invariant>> =
            vec![Box::new(MonotonicInvariant::new("counter value", value_fn))];

        let config = FuzzConfig {
            seed: 7,
            max_runs: 200,
            sequence_depth: 8,
            actors: vec![[1u8; 20]],
        };

        let violation = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        )
        .expect("MockBackend's decrement() is a real bug — the fuzzer must find it");

        assert_eq!(violation.invariant_name, "counter value");
        assert!(
            violation.sequence.len() <= 3,
            "expected shrinking to collapse this down to a handful of calls, got {}",
            violation.sequence.len()
        );
        // The shrunk reproduction must still actually reproduce when replayed.
        let mut replay = MockBackend::default();
        let invariants: Vec<Box<dyn Invariant>> = vec![Box::new(MonotonicInvariant::new(
            "counter value",
            testing::counter_functions()
                .into_iter()
                .find(|f| f.selector == testing::VALUE)
                .unwrap(),
        ))];
        assert!(run_sequence(&mut replay, &violation.sequence, &invariants).is_some());
    }

    #[test]
    fn catches_conservation_violation() {
        let functions = testing::token_functions();
        let total_supply_fn = functions
            .iter()
            .find(|f| f.selector == testing::TOTAL_SUPPLY)
            .unwrap()
            .clone();
        let balance_of_fn = functions
            .iter()
            .find(|f| f.selector == testing::BALANCE_OF)
            .unwrap()
            .clone();
        let actors = vec![[1u8; 20], [2u8; 20], [3u8; 20]];
        let invariants: Vec<Box<dyn Invariant>> = vec![Box::new(ConservationInvariant::new(
            "token conservation",
            total_supply_fn,
            balance_of_fn,
            actors.clone(),
        ))];

        let config = FuzzConfig {
            seed: 99,
            max_runs: 300,
            sequence_depth: 8,
            actors,
        };

        let violation = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        )
        .expect("MockBackend's buggyCredit() breaks conservation — the fuzzer must find it");

        assert_eq!(violation.invariant_name, "token conservation");
        assert!(violation.message.contains("sum(balanceOf)"));
    }

    #[test]
    fn no_violation_when_only_the_safe_function_is_available() {
        // Sanity check against false positives: a run that can only call
        // the correct `mint` (never the buggy credit path) must never
        // report a conservation violation.
        let mint_only = vec![testing::token_functions()
            .into_iter()
            .find(|f| f.selector == testing::MINT)
            .unwrap()];
        let mut all = mint_only;
        all.push(
            testing::token_functions()
                .into_iter()
                .find(|f| f.selector == testing::TOTAL_SUPPLY)
                .unwrap(),
        );
        all.push(
            testing::token_functions()
                .into_iter()
                .find(|f| f.selector == testing::BALANCE_OF)
                .unwrap(),
        );

        let total_supply_fn = all
            .iter()
            .find(|f| f.selector == testing::TOTAL_SUPPLY)
            .unwrap()
            .clone();
        let balance_of_fn = all
            .iter()
            .find(|f| f.selector == testing::BALANCE_OF)
            .unwrap()
            .clone();
        let actors = vec![[1u8; 20], [2u8; 20]];
        let invariants: Vec<Box<dyn Invariant>> = vec![Box::new(ConservationInvariant::new(
            "token conservation",
            total_supply_fn,
            balance_of_fn,
            actors.clone(),
        ))];

        let config = FuzzConfig {
            seed: 5,
            max_runs: 150,
            sequence_depth: 8,
            actors,
        };

        let result = fuzz(
            || Box::new(MockBackend::default()),
            &all,
            invariants,
            config,
        );
        assert!(
            result.is_none(),
            "mint-only sequences must never violate conservation"
        );
    }

    #[test]
    fn catches_access_control_violation() {
        let functions = testing::ownable_functions();
        let owner_fn = functions
            .iter()
            .find(|f| f.selector == testing::OWNER)
            .unwrap()
            .clone();
        let transfer_fn = functions
            .iter()
            .find(|f| f.selector == testing::TRANSFER_OWNERSHIP)
            .unwrap()
            .clone();

        let invariants: Vec<Box<dyn Invariant>> = vec![Box::new(AccessControlInvariant::new(
            "ownership transfer authorization",
            owner_fn,
            transfer_fn,
        ))];

        // Actor pool deliberately excludes testing::GENESIS_OWNER, so any
        // successful transferOwnership in this run is necessarily from a
        // non-owner — MockBackend's TRANSFER_OWNERSHIP has no caller check
        // at all, so this must always find a violation.
        let config = FuzzConfig {
            seed: 3,
            max_runs: 50,
            sequence_depth: 4,
            actors: vec![[1u8; 20], [2u8; 20]],
        };

        let violation = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        )
        .expect("MockBackend's transferOwnership has no caller check — the fuzzer must find it");

        assert_eq!(violation.invariant_name, "ownership transfer authorization");
        assert!(violation.message.contains("non-owner"));
    }

    #[test]
    fn no_false_positive_when_ownership_transfer_is_actually_guarded() {
        let functions = testing::ownable_functions_safe();
        let owner_fn = functions
            .iter()
            .find(|f| f.selector == testing::OWNER)
            .unwrap()
            .clone();
        let transfer_fn = functions
            .iter()
            .find(|f| f.selector == testing::SAFE_TRANSFER_OWNERSHIP)
            .unwrap()
            .clone();

        let invariants: Vec<Box<dyn Invariant>> = vec![Box::new(AccessControlInvariant::new(
            "ownership transfer authorization",
            owner_fn,
            transfer_fn,
        ))];

        let config = FuzzConfig {
            seed: 11,
            max_runs: 150,
            sequence_depth: 6,
            actors: vec![[1u8; 20], [2u8; 20]],
        };

        let result = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        );
        assert!(
            result.is_none(),
            "a correctly owner-gated transferOwnership must never be flagged"
        );
    }

    #[test]
    fn catches_reentrancy_from_the_call_trace() {
        let functions = testing::vault_functions_vulnerable();
        let invariants: Vec<Box<dyn Invariant>> =
            vec![Box::new(ReentrancyInvariant::new("reentrancy"))];

        let config = FuzzConfig {
            seed: 1,
            max_runs: 20,
            sequence_depth: 3,
            actors: vec![[1u8; 20], [2u8; 20]],
        };

        let violation = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        )
        .expect("the vulnerable withdraw's trace shows exploitable reentrancy — must be caught");

        assert_eq!(violation.invariant_name, "reentrancy");
        assert!(violation.message.contains("re-entered"));
    }

    #[test]
    fn no_reentrancy_false_positive_on_guarded_or_cei_vaults() {
        let functions = testing::vault_functions_safe();
        let invariants: Vec<Box<dyn Invariant>> =
            vec![Box::new(ReentrancyInvariant::new("reentrancy"))];

        let config = FuzzConfig {
            seed: 4,
            max_runs: 200,
            sequence_depth: 6,
            actors: vec![[1u8; 20], [2u8; 20]],
        };

        let result = fuzz(
            || Box::new(MockBackend::default()),
            &functions,
            invariants,
            config,
        );
        assert!(
            result.is_none(),
            "guarded and CEI-correct withdraws must never be flagged as reentrancy"
        );
    }
}
