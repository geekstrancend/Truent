//! Chain-agnostic semantic-model extraction for Soroban contracts.
//!
//! Builds a [`truent_ir::SemanticModel`] from the same
//! [`crate::soroban_parser::parse_contract_functions`] facts the detectors
//! use, so this feeds the shared
//! [`truent_ir::rules::find_unauthorized_privileged_mutations`] rule the
//! same way `truent_analyzer_solana`/`_evm`/`_move` do — a `require_auth()`
//! call is Soroban's analogue of Solana's `Signer<'info>`/Move's `&signer`.

use crate::soroban_parser::parse_contract_functions;
use truent_ir::{
    AuthCheckKind, AuthorizationCheck, MutationKind, PrivilegedMutation, SemanticModel,
};

/// Function-name substrings that indicate a privileged mutation, paired
/// with the mutation category they represent.
const SENSITIVE_HANDLERS: &[(&str, MutationKind)] = &[
    ("withdraw", MutationKind::FundTransfer),
    ("transfer", MutationKind::FundTransfer),
    ("pay", MutationKind::FundTransfer),
    ("mint", MutationKind::FundTransfer),
    ("burn", MutationKind::FundTransfer),
    ("set_admin", MutationKind::AuthorityChange),
    ("set_owner", MutationKind::AuthorityChange),
    ("remove", MutationKind::AccountClose),
];

/// Build a chain-agnostic semantic model from Soroban contract source.
pub fn build_semantic_model(source: &str, file_path: &str) -> anyhow::Result<SemanticModel> {
    let functions = parse_contract_functions(source)?;
    let mut model = SemanticModel::new("soroban", file_path);

    for f in &functions {
        let lower = f.name.to_lowercase();
        let kind =
            if let Some((_, kind)) = SENSITIVE_HANDLERS.iter().find(|(n, _)| lower.contains(n)) {
                kind.clone()
            } else if f.upgrades_wasm {
                MutationKind::Upgrade
            } else {
                continue;
            };

        let guards = if f.has_require_auth {
            vec![AuthorizationCheck {
                kind: AuthCheckKind::Signer,
                source: "require_auth".to_string(),
            }]
        } else {
            Vec::new()
        };

        model.mutations.push(PrivilegedMutation {
            entry_point: f.name.clone(),
            kind,
            line: f.line,
            guards,
        });
    }

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use truent_ir::rules::find_unauthorized_privileged_mutations;

    const FIXTURE: &str = r#"
#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn withdraw(env: Env, to: Address, amount: i128) {
        env.storage().persistent().set(&DataKey::Balance(to), &amount);
    }

    pub fn admin_withdraw(env: Env, admin: Address, to: Address, amount: i128) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Balance(to), &amount);
    }
}
"#;

    #[test]
    fn flags_withdraw_with_no_guard_but_not_admin_withdraw() {
        let model = build_semantic_model(FIXTURE, "vault.rs").unwrap();
        assert_eq!(model.chain, "soroban");
        assert_eq!(model.mutations.len(), 2);

        let findings = find_unauthorized_privileged_mutations(&model);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("withdraw"));
        assert!(!findings[0].message.contains("admin_withdraw"));
    }
}
