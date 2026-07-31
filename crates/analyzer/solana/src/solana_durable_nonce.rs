//! Detector for missing durable nonce validation in Solana programs.
//!
//! This detector identifies Solana programs that read durable nonces without
//! proper validation or increment. This pattern was exploited in H46 Drift Protocol
//! ($285M, 2026) where transaction replay attacks were possible.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    /// Regex to match durable nonce read operations
    static ref NONCE_READ_REGEX: Regex =
        Regex::new(r"(?i)(nonce|durable|replay|authorized)").unwrap();

    /// Regex to match instruction handler functions
    static ref IX_HANDLER_REGEX: Regex =
        Regex::new(r"(?i)(pub fn|fn)\s+\w*(?:process|handle|execute)\w*\s*\(").unwrap();
}

/// Detects missing durable nonce validation in Solana programs.
pub fn detect_solana_durable_nonce_validation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Pattern 1: Find all instruction handlers
    for (handler_line_num, handler_line) in source.lines().enumerate() {
        if !IX_HANDLER_REGEX.is_match(handler_line) {
            continue;
        }

        let handler_name = extract_handler_name(handler_line);

        // Extract handler body (~60 lines)
        let handler_start = handler_line_num;
        let handler_end = (handler_line_num + 60).min(source.lines().count());
        let handler_body = source
            .lines()
            .skip(handler_start)
            .take(handler_end - handler_start)
            .collect::<Vec<&str>>()
            .join("\n");

        // Pattern 2: Check if nonce field is read
        if reads_nonce_field(&handler_body) {
            // Pattern 3: Check if nonce is validated after read
            if !validates_nonce_after_read(&handler_body) {
                let message = format!(
                    "Durable nonce read in '{}' but not validated. This allows transaction replay attacks. \
                     H46 Drift Protocol ($285M) was exploited when nonce validation was missing, \
                     allowing attackers to replay liquidation transactions or withdrawal requests. \
                     \
                     Attack: \
                     1. Intercept valid transaction with nonce \
                     2. Replay same transaction (tx signature + nonce still valid) \
                     3. Execute action twice (liquidation, withdrawal, etc.) \
                     \
                     Required fix: Validate nonce BEFORE processing: \
                     require eq ctx.accounts.nonce_account.data_is_empty == false \
                     require eq ctx.accounts.nonce_account.lamports > 0",
                    handler_name
                );

                findings.push(
                    Finding::new(
                        "sol_durable_nonce_validation".to_string(),
                        truent_core::Severity::Critical,
                        file_path.to_string(),
                        handler_line_num + 1,
                        0,
                        message,
                        handler_line.trim().to_string(),
                    )
                    .with_metadata("exploit_id".to_string(), "H46".to_string())
                    .with_metadata("exploit_name".to_string(), "Drift Protocol".to_string())
                    .with_metadata("loss".to_string(), "$285M".to_string())
                    .with_metadata("year".to_string(), "2026".to_string())
                    .with_metadata("chain".to_string(), "Solana".to_string())
                    .with_metadata(
                        "vulnerability_type".to_string(),
                        "replay_attack".to_string(),
                    )
                    .with_metadata("detector".to_string(), "nonce_validation_check".to_string())
                    .with_source_fragment(handler_body.clone()),
                );
            }

            // Pattern 4: Check if nonce is NOT incremented after validation
            if validates_nonce_after_read(&handler_body)
                && !increments_nonce_after_validation(&handler_body)
            {
                let message = format!(
                    "Durable nonce validated in '{}' but never incremented. \
                     Must increment nonce atomically after validation to prevent replay. \
                     Without incrementing, the same nonce can be reused for multiple transactions.",
                    handler_name
                );

                findings.push(
                    Finding::new(
                        "sol_durable_nonce_validation".to_string(),
                        truent_core::Severity::High,
                        file_path.to_string(),
                        handler_line_num + 1,
                        0,
                        message,
                        handler_line.trim().to_string(),
                    )
                    .with_metadata("exploit_id".to_string(), "H46".to_string())
                    .with_metadata(
                        "vulnerability_type".to_string(),
                        "nonce_not_incremented".to_string(),
                    )
                    .with_metadata("detector".to_string(), "nonce_increment_check".to_string()),
                );
            }
        }
    }

    findings
}

/// Extract handler name from function definition
fn extract_handler_name(line: &str) -> String {
    if let Some(fn_pos) = line.find("fn ") {
        let after_fn = &line[fn_pos + 3..];
        if let Some(paren_pos) = after_fn.find('(') {
            return after_fn[..paren_pos].trim().to_string();
        }
    }
    "handler".to_string()
}

/// Check if handler body reads nonce field
fn reads_nonce_field(handler_body: &str) -> bool {
    let handler_lower = handler_body.to_lowercase();
    handler_lower.contains("nonce")
        || handler_lower.contains("durable")
        || handler_lower.contains("authorized")
}

/// Check if nonce is validated after being read
fn validates_nonce_after_read(handler_body: &str) -> bool {
    let handler_lower = handler_body.to_lowercase();

    // Look for validation patterns
    handler_lower.contains("require")
        || handler_lower.contains("assert")
        || handler_lower.contains("is_signer")
        || handler_lower.contains("verify")
        || handler_lower.contains("validate_nonce")
}

/// Check if nonce is incremented after validation
fn increments_nonce_after_validation(handler_body: &str) -> bool {
    let handler_lower = handler_body.to_lowercase();

    // Look for nonce increment patterns
    handler_lower.contains("nonce_account.try_borrow_mut_data")
        || handler_lower.contains("nonce += 1")
        || handler_lower.contains("nonce_counter")
        || handler_lower.contains("next_nonce")
        || handler_lower.contains("advance_nonce")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerable_missing_nonce_check() {
        let code = r#"
            pub fn process_liquidation(
                program_id: &Pubkey,
                accounts: &[AccountInfo],
                instruction_data: &[u8],
            ) -> ProgramResult {
                let account_info_iter = &mut accounts.iter();
                let nonce_account = next_account_info(account_info_iter)?;
                
                // VULNERABLE: Reads nonce but never validates!
                let nonce = u64::from_le_bytes(nonce_account.data[..8].try_into()?);
                
                // Process liquidation with unvalidated nonce
                process_liquidation_with_nonce(nonce)?;
                
                Ok(())
            }
        "#;

        let findings = detect_solana_durable_nonce_validation(code, "drift.rs");
        assert!(
            !findings.is_empty(),
            "Should detect missing nonce validation"
        );
        assert!(findings[0].invariant_id.contains("durable_nonce"));
    }

    #[test]
    fn test_vulnerable_validated_but_not_incremented() {
        let code = r#"
            pub fn process_transaction(
                program_id: &Pubkey,
                accounts: &[AccountInfo],
                instruction_data: &[u8],
            ) -> ProgramResult {
                let account_info_iter = &mut accounts.iter();
                let nonce_account = next_account_info(account_info_iter)?;
                
                let nonce = u64::from_le_bytes(nonce_account.data[..8].try_into()?);
                
                // Validates nonce
                require_signer(nonce_account)?;
                
                // VULNERABLE: Never increments nonce!
                // Same nonce can be replayed multiple times
                
                Ok(())
            }
        "#;

        let findings = detect_solana_durable_nonce_validation(code, "drift.rs");
        // Should get finding about nonce not being incremented
        assert!(findings.iter().any(|f| f.message.contains("increment")));
    }

    #[test]
    fn test_safe_with_nonce_validation_and_increment() {
        let code = r#"
            pub fn process_transaction(
                program_id: &Pubkey,
                accounts: &[AccountInfo],
                instruction_data: &[u8],
            ) -> ProgramResult {
                let account_info_iter = &mut accounts.iter();
                let nonce_account = next_account_info(account_info_iter)?;
                
                // Validate nonce
                require_signer(nonce_account)?;
                
                let mut nonce = u64::from_le_bytes(nonce_account.data[..8].try_into()?);
                
                // Process transaction
                execute_transaction(nonce)?;
                
                // SAFE: Increment nonce for next transaction
                nonce += 1;
                nonce_account.try_borrow_mut_data()?[..8].copy_from_slice(&nonce.to_le_bytes());
                
                Ok(())
            }
        "#;

        let findings = detect_solana_durable_nonce_validation(code, "safe.rs");
        assert!(
            findings.is_empty(),
            "Should not flag with proper nonce handling"
        );
    }
}
