#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Dynamic invariant fuzzing for Solana programs.
//!
//! The Solana counterpart to `truent-dynamic-evm`. Solana's execution model
//! is account-based, not the EVM's flat-calldata single-caller shape, so this
//! crate defines its own call model (`model`), invariant oracles
//! (`invariant`), instruction generator (`generator`), and fuzz/shrink loop
//! (`fuzz`) rather than forcing the EVM types onto it. The engine logic is
//! proven against an in-memory mock program (`testing`, `cfg(test)`).
//!
//! A real execution backend (running compiled BPF bytecode on an in-process
//! Solana VM) is not part of this release: the VM's dependencies carry
//! unpatched RUSTSEC advisories, and a security tool must not ship
//! known-vulnerable crypto. It is tracked for a later version; the backend
//! source lives in git history at the 0.3.x line.

pub mod backend;
pub mod config;
pub mod fuzz;
pub mod generator;
pub mod idl;
pub mod invariant;
pub mod model;

#[cfg(test)]
mod testing;

pub use backend::SvmBackend;
pub use config::{parse_plan, ConfigError, FuzzPlan, GenesisAccount};
pub use fuzz::{format_poc, fuzz, run_sequence, shrink, FuzzConfig, Step, Violation};
pub use generator::AccountPool;
pub use idl::{anchor_discriminator, parse_idl, IdlError, IdlProgram, SkippedInstruction};
pub use invariant::{AccountOwnerInvariant, SolanaInvariant, TokenConservationInvariant};
pub use model::{
    AccountMeta, AccountRole, ArgKind, Instruction, InstructionSpec, IxOutcome, Pubkey,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        token_functions_safe, token_functions_vulnerable, MockSvm, AMOUNT_OFFSET, MINT,
        SUPPLY_OFFSET, TOKEN_ACCT_1, TOKEN_ACCT_2, TOKEN_ACCT_3, TOKEN_PROGRAM,
    };

    fn pool() -> AccountPool {
        AccountPool::new(
            vec![crate::testing::AUTHORITY],
            vec![TOKEN_ACCT_1, TOKEN_ACCT_2, TOKEN_ACCT_3, MINT],
            vec![],
        )
    }

    fn conservation() -> Box<dyn SolanaInvariant> {
        Box::new(TokenConservationInvariant::new(
            "SPL conservation: sum(amounts) == mint supply",
            MINT,
            vec![TOKEN_ACCT_1, TOKEN_ACCT_2, TOKEN_ACCT_3],
            AMOUNT_OFFSET,
            SUPPLY_OFFSET,
        ))
    }

    #[test]
    fn catches_token_conservation_violation_and_shrinks_it() {
        let specs = token_functions_vulnerable();
        let invariants: Vec<Box<dyn SolanaInvariant>> = vec![conservation()];
        let config = FuzzConfig {
            seed: 1,
            max_runs: 300,
            sequence_depth: 8,
        };

        let violation = fuzz(
            || Box::new(MockSvm::default()),
            TOKEN_PROGRAM,
            &specs,
            &pool(),
            invariants,
            config,
        )
        .expect("the buggy airdrop credits without updating supply — the fuzzer must catch it");

        assert!(violation.invariant_name.contains("conservation"));
        assert!(violation.message.contains("mint supply"));
        // Shrinking must collapse to a small reproducer, and the airdrop
        // instruction (disc 0x11) must be the culprit that remains.
        assert!(
            violation.sequence.len() <= 3,
            "expected a minimal reproduction, got {} steps",
            violation.sequence.len()
        );
        assert!(
            violation
                .sequence
                .iter()
                .any(|(ix, _)| ix.data.first() == Some(&crate::testing::IX_AIRDROP)),
            "the airdrop instruction must be in the minimal reproduction"
        );
    }

    #[test]
    fn no_false_positive_when_only_the_correct_mint_is_available() {
        let specs = token_functions_safe();
        let invariants: Vec<Box<dyn SolanaInvariant>> = vec![conservation()];
        let config = FuzzConfig {
            seed: 7,
            max_runs: 300,
            sequence_depth: 10,
        };

        let result = fuzz(
            || Box::new(MockSvm::default()),
            TOKEN_PROGRAM,
            &specs,
            &pool(),
            invariants,
            config,
        );
        assert!(
            result.is_none(),
            "the correct mint updates supply in lockstep — conservation must hold"
        );
    }

    #[test]
    fn catches_unchecked_owner_reassignment() {
        // A program surface whose only instruction reassigns an account's
        // owner with no authority check must trip the owner-integrity
        // invariant.
        let specs = vec![InstructionSpec::new(
            "reassign",
            vec![crate::testing::IX_REASSIGN_UNCHECKED],
            vec![ArgKind::Pubkey],
            vec![AccountRole::Writable],
            true,
        )];
        let invariants: Vec<Box<dyn SolanaInvariant>> = vec![Box::new(
            AccountOwnerInvariant::track("owner integrity", TOKEN_ACCT_1),
        )];
        let config = FuzzConfig {
            seed: 3,
            max_runs: 200,
            sequence_depth: 4,
        };

        let violation = fuzz(
            || Box::new(MockSvm::default()),
            TOKEN_PROGRAM,
            &specs,
            &AccountPool::new(vec![crate::testing::AUTHORITY], vec![TOKEN_ACCT_1], vec![]),
            invariants,
            config,
        )
        .expect("unchecked owner reassignment must be caught");
        assert!(violation.message.contains("owner changed"));
    }
}
