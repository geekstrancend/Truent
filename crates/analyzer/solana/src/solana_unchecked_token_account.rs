//! Solana Unchecked Token/Mint Account Detector
//!
//! Detects H59 vulnerability: an Anchor account field whose name suggests it
//! holds a token/mint/collateral/vault/pool reference, typed as a raw
//! `AccountInfo`/`UncheckedAccount` with no Anchor-enforced typing
//! (`Account<'info, TokenAccount>`/`InterfaceAccount`) and no owner/address/
//! seeds constraint narrowing it down. This is the exact shape behind
//! Cashio ($52M, Mar 2022) and Crema Finance ($8.8M, Jul 2022): both let an
//! attacker substitute a completely fake account (a fake mint, a fake
//! concentrated-liquidity tick account) because nothing on-chain verified
//! the account was really the mint/pool/collateral it claimed to be.

use crate::anchor_model::AccountSecurity;
use crate::anchor_parser::parse_anchor_accounts;
use truent_core::{Finding, Severity};

/// Field-name substrings suggesting this account is a token/mint/collateral
/// reference, where a fake-account substitution would let an attacker mint
/// or withdraw against worthless collateral.
const TOKEN_LIKE_NAMES: &[&str] = &[
    "mint",
    "collateral",
    "vault",
    "token",
    "reserve",
    "pool",
    "deposit",
    "tick",
];

pub fn detect_unchecked_token_account_type(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let Ok(structs) = parse_anchor_accounts(source, file_path) else {
        return findings;
    };

    for s in &structs {
        for field in &s.fields {
            let lower = field.name.to_lowercase();
            if !TOKEN_LIKE_NAMES.iter().any(|n| lower.contains(n)) {
                continue;
            }

            let is_risky = match &field.security {
                AccountSecurity::TrulyUnchecked => true,
                AccountSecurity::ConstrainedUnchecked { .. } => {
                    !field.security.is_adequately_constrained()
                }
                _ => false,
            };
            if !is_risky {
                continue;
            }

            findings.push(
                Finding::new(
                    "sol_unchecked_token_account_type".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    field.line_number,
                    0,
                    format!(
                        "Account field '{}' in '{}' looks like a token/mint/collateral \
                         reference but is a raw AccountInfo/UncheckedAccount with no owner/\
                         address/seeds constraint — an attacker can substitute a fake account, \
                         the exact pattern behind Cashio ($52M, Mar 2022) and Crema Finance \
                         ($8.8M, Jul 2022)",
                        field.name, s.name
                    ),
                    field.name.clone(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_unchecked_mint_account() {
        let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MintCollateral<'info> {
    pub authority: Signer<'info>,
    pub collateral_mint: AccountInfo<'info>,
}
"#;
        let findings = detect_unchecked_token_account_type(source, "cashio.rs");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "sol_unchecked_token_account_type"));
    }

    #[test]
    fn does_not_flag_typed_token_account() {
        let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MintCollateral<'info> {
    pub authority: Signer<'info>,
    pub collateral_mint: Account<'info, Mint>,
}
"#;
        let findings = detect_unchecked_token_account_type(source, "safe.rs");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "sol_unchecked_token_account_type"));
    }

    #[test]
    fn does_not_flag_constrained_account() {
        let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MintCollateral<'info> {
    pub authority: Signer<'info>,
    #[account(address = KNOWN_MINT)]
    pub collateral_mint: AccountInfo<'info>,
}
"#;
        let findings = detect_unchecked_token_account_type(source, "constrained.rs");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "sol_unchecked_token_account_type"));
    }

    #[test]
    fn does_not_flag_unrelated_account_names() {
        let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub authority: Signer<'info>,
    pub metadata: AccountInfo<'info>,
}
"#;
        let findings = detect_unchecked_token_account_type(source, "unrelated.rs");
        assert!(findings.is_empty());
    }
}
