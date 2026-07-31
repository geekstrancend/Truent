//! Solana Fake Sysvar Instructions Account Detector
//!
//! Detects H61 vulnerability: reading the Instructions sysvar via the
//! unchecked `load_instruction_at` helper (rather than
//! `load_instruction_at_checked`, or an explicit manual key comparison
//! against `sysvar::instructions::ID`).
//!
//! This is the exact root cause of the Wormhole Solana bridge hack ($326M,
//! Feb 2022, still the second-largest DeFi exploit ever): the program
//! accepted an attacker-supplied fake account in place of the real
//! Instructions sysvar and never verified the account's address, so a
//! forged "guardian signature" instruction was read back and trusted as if
//! `secp256k1` verification had actually happened. `load_instruction_at_checked`
//! (added after this hack) performs that address check internally — plain
//! `load_instruction_at` does not.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref UNCHECKED_LOAD: Regex = Regex::new(r"(?i)\bload_instruction_at\s*\(").unwrap();
    static ref CHECKED_LOAD_OR_MANUAL_ID_CHECK: Regex = Regex::new(
        r"(?i)load_instruction_at_checked|instructions::ID|sysvar::instructions::ID|check_id\s*\("
    )
    .unwrap();
}

pub fn detect_fake_sysvar_instruction_account(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        // `load_instruction_at_checked` also matches `load_instruction_at\s*\(`
        // as a substring, so exclude lines that are actually the checked
        // variant before treating this as the unchecked call.
        if !UNCHECKED_LOAD.is_match(line) || line.to_lowercase().contains("_checked") {
            continue;
        }
        if CHECKED_LOAD_OR_MANUAL_ID_CHECK.is_match(source) {
            continue;
        }

        findings.push(
            Finding::new(
                "sol_fake_sysvar_instruction_account".to_string(),
                Severity::Critical,
                file_path.to_string(),
                line_num + 1,
                0,
                "load_instruction_at() reads the Instructions sysvar without verifying the \
                 supplied account's address, letting a fake account be substituted for the \
                 real sysvar — the exact root cause of the Wormhole Solana bridge hack \
                 ($326M, Feb 2022). Use load_instruction_at_checked() or explicitly compare \
                 the account key against sysvar::instructions::ID"
                    .to_string(),
                line.trim().to_string(),
            )
            .with_metadata("chain".to_string(), "solana".to_string()),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_unchecked_load_instruction_at() {
        let source = r#"
pub fn verify_signatures(ctx: Context<VerifySignatures>) -> Result<()> {
    let ix = load_instruction_at(0, &ctx.accounts.instructions.to_account_info().data.borrow())?;
    Ok(())
}
"#;
        let findings = detect_fake_sysvar_instruction_account(source, "wormhole.rs");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "sol_fake_sysvar_instruction_account"));
    }

    #[test]
    fn does_not_flag_checked_variant() {
        let source = r#"
pub fn verify_signatures(ctx: Context<VerifySignatures>) -> Result<()> {
    let ix = load_instruction_at_checked(0, &ctx.accounts.instructions.to_account_info())?;
    Ok(())
}
"#;
        let findings = detect_fake_sysvar_instruction_account(source, "safe.rs");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "sol_fake_sysvar_instruction_account"));
    }

    #[test]
    fn does_not_flag_when_manual_id_check_present() {
        let source = r#"
pub fn verify_signatures(ctx: Context<VerifySignatures>) -> Result<()> {
    require!(ctx.accounts.instructions.key() == sysvar::instructions::ID, ErrorCode::InvalidSysvar);
    let ix = load_instruction_at(0, &ctx.accounts.instructions.to_account_info().data.borrow())?;
    Ok(())
}
"#;
        let findings = detect_fake_sysvar_instruction_account(source, "manual_check.rs");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "sol_fake_sysvar_instruction_account"));
    }
}
