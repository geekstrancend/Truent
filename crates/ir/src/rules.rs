//! Chain-agnostic detection rules operating purely over [`crate::semantic::SemanticModel`].
//!
//! A rule in this module contains zero chain-specific code — it is written
//! once against the shared IR and applies unmodified to every chain whose
//! analyzer populates a [`SemanticModel`]. This is the mechanism Epic 6.1
//! exists to prove out: adding a new chain means writing an IR-populating
//! extractor, not re-deriving every rule.

use crate::semantic::SemanticModel;
use truent_core::{Finding, Severity};

/// The shared invariant ID for [`find_unauthorized_privileged_mutations`],
/// reused verbatim by every chain that reports it.
pub const UNAUTHORIZED_PRIVILEGED_MUTATION: &str = "unauthorized_privileged_mutation";

/// Flags privileged mutations (fund transfers, authority changes, upgrades,
/// account closes) reachable through an entry point with no authorization
/// check guarding them.
///
/// This is chain-agnostic: it only reads [`SemanticModel::mutations`] and
/// each mutation's guard list, which every chain-specific extractor is
/// responsible for populating from its own native syntax.
pub fn find_unauthorized_privileged_mutations(model: &SemanticModel) -> Vec<Finding> {
    model
        .mutations
        .iter()
        .filter(|m| !m.is_guarded())
        .map(|m| {
            Finding::new(
                UNAUTHORIZED_PRIVILEGED_MUTATION.to_string(),
                Severity::Critical,
                model.source_path.clone(),
                m.line,
                0,
                format!(
                    "Entry point '{}' performs a privileged mutation ({:?}) with no \
                     authorization check reaching it",
                    m.entry_point, m.kind
                ),
                m.entry_point.clone(),
            )
            .with_metadata("chain".to_string(), model.chain.clone())
            .with_metadata("mutation_kind".to_string(), format!("{:?}", m.kind))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AuthCheckKind, AuthorizationCheck, MutationKind, PrivilegedMutation};

    #[test]
    fn flags_unguarded_mutation() {
        let mut model = SemanticModel::new("evm", "Vault.sol");
        model.mutations.push(PrivilegedMutation {
            entry_point: "withdraw".to_string(),
            kind: MutationKind::FundTransfer,
            line: 10,
            guards: vec![],
        });

        let findings = find_unauthorized_privileged_mutations(&model);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].invariant_id, UNAUTHORIZED_PRIVILEGED_MUTATION);
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[0].metadata.get("chain").unwrap(), "evm");
    }

    #[test]
    fn does_not_flag_guarded_mutation() {
        let mut model = SemanticModel::new("evm", "Vault.sol");
        model.mutations.push(PrivilegedMutation {
            entry_point: "withdraw".to_string(),
            kind: MutationKind::FundTransfer,
            line: 10,
            guards: vec![AuthorizationCheck {
                kind: AuthCheckKind::Signer,
                source: "onlyOwner".to_string(),
            }],
        });

        assert!(find_unauthorized_privileged_mutations(&model).is_empty());
    }
}
