//! Chain-agnostic semantic-model extraction for Solana/Anchor programs.
//!
//! Builds a [`truent_ir::SemanticModel`] from Anchor source by reusing the
//! real `syn`-based account parser in [`crate::anchor_parser`] — not
//! line-matching — so guards are derived from the actual Anchor account
//! security posture ([`crate::anchor_model::AccountSecurity`]), the same
//! model Epic 6.2 wires the rest of the Solana detectors onto.

use crate::anchor_model::{AccountSecurity, AnchorAccountStruct, AnchorConstraint};
use crate::anchor_parser::parse_anchor_accounts;
use regex::Regex;
use truent_ir::{
    AuthCheckKind, AuthorizationCheck, MutationKind, PrivilegedMutation, SemanticModel,
};
use std::collections::BTreeMap;

/// Instruction handler name substrings that indicate a privileged mutation,
/// paired with the mutation category they represent.
const SENSITIVE_HANDLERS: &[(&str, MutationKind)] = &[
    ("withdraw", MutationKind::FundTransfer),
    ("transfer", MutationKind::FundTransfer),
    ("disburse", MutationKind::FundTransfer),
    ("close", MutationKind::AccountClose),
    ("set_authority", MutationKind::AuthorityChange),
    ("upgrade", MutationKind::Upgrade),
    // Broadened after auditing Cashio ($52M, Mar 2022): its exploited entry
    // point was a mint/deposit handler, not one of the verbs above, so it
    // would have slipped past this list entirely despite its accounts
    // struct having a completely unchecked "collateral" field.
    ("mint", MutationKind::FundTransfer),
    ("deposit", MutationKind::FundTransfer),
    ("collateral", MutationKind::FundTransfer),
    ("borrow", MutationKind::FundTransfer),
    ("liquidate", MutationKind::FundTransfer),
    ("swap", MutationKind::FundTransfer),
];

/// Build a chain-agnostic semantic model from Anchor program source.
///
/// Instruction handlers (`pub fn <name>(ctx: Context<Struct>, ...)`) whose
/// name matches a sensitive pattern become [`PrivilegedMutation`] entries;
/// their guards are read off the associated `#[derive(Accounts)]` struct's
/// parsed [`AccountSecurity`] for each field, so a real `Signer<'info>` or a
/// `has_one`/`owner`/`address` constraint is recognized as an authorization
/// check rather than re-derived from source text.
pub fn build_semantic_model(source: &str, file_path: &str) -> anyhow::Result<SemanticModel> {
    let structs = parse_anchor_accounts(source, file_path)?;
    let structs_by_name: BTreeMap<&str, &AnchorAccountStruct> =
        structs.iter().map(|s| (s.name.as_str(), s)).collect();

    let mut model = SemanticModel::new("solana", file_path);

    let handler_re =
        Regex::new(r"pub\s+fn\s+(\w+)\s*\(\s*ctx\s*:\s*Context<(?:'\w+\s*,\s*)?(\w+)>")
            .expect("valid regex");

    for (line_idx, line) in source.lines().enumerate() {
        let Some(caps) = handler_re.captures(line) else {
            continue;
        };
        let handler_name = &caps[1];
        let struct_name = &caps[2];

        let lower = handler_name.to_lowercase();
        let Some((_, kind)) = SENSITIVE_HANDLERS
            .iter()
            .find(|(needle, _)| lower.contains(needle))
        else {
            continue;
        };

        let guards = structs_by_name
            .get(struct_name)
            .map(|s| extract_guards(s))
            .unwrap_or_default();

        model.mutations.push(PrivilegedMutation {
            entry_point: handler_name.to_string(),
            kind: kind.clone(),
            line: line_idx + 1,
            guards,
        });
    }

    Ok(model)
}

/// Read authorization guards off an Anchor accounts struct's parsed field
/// security. Only checks that are actually enforced by the framework or an
/// explicit constraint count — a bare `/// CHECK:` comment documents a
/// developer's manual reasoning about account *safety*, not who is
/// authorized to call the instruction, so it is intentionally not treated
/// as an authorization guard here.
fn extract_guards(s: &AnchorAccountStruct) -> Vec<AuthorizationCheck> {
    s.fields
        .iter()
        .filter_map(|f| match &f.security {
            AccountSecurity::AnchorSigner => Some(AuthorizationCheck {
                kind: AuthCheckKind::Signer,
                source: f.name.clone(),
            }),
            AccountSecurity::ConstrainedUnchecked { constraints }
                if constraints.iter().any(|c| {
                    matches!(
                        c,
                        AnchorConstraint::Owner(_)
                            | AnchorConstraint::Address(_)
                            | AnchorConstraint::HasOne(_)
                    )
                }) =>
            {
                Some(AuthorizationCheck {
                    kind: AuthCheckKind::RoleOrCapability,
                    source: f.name.clone(),
                })
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use truent_ir::rules::find_unauthorized_privileged_mutations;

    /// Real, minimal Anchor program: `withdraw` has no signer or ownership
    /// constraint anywhere in its accounts struct; `admin_withdraw` is
    /// gated by a `has_one` constraint plus an explicit `Signer<'info>`.
    const FIXTURE: &str = r#"
use anchor_lang::prelude::*;

#[program]
pub mod vault {
    use super::*;

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn admin_withdraw(ctx: Context<AdminWithdraw>, amount: u64) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AdminWithdraw<'info> {
    #[account(mut, has_one = authority)]
    pub vault: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
"#;

    #[test]
    fn flags_withdraw_with_no_guard_but_not_admin_withdraw() {
        let model = build_semantic_model(FIXTURE, "vault.rs").unwrap();
        assert_eq!(model.chain, "solana");
        assert_eq!(model.mutations.len(), 2);

        let findings = find_unauthorized_privileged_mutations(&model);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("withdraw"));
        assert!(!findings[0].message.contains("admin_withdraw"));
    }
}
