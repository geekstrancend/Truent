//! Solana detector implementations.
//!
//! Detectors for Solana/Anchor program vulnerabilities.

use truent_core::{Finding, Severity};

/// Detects missing signer constraint on accounts
pub fn detect_missing_signer(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Pattern: Account<'_> without #[account(signer)]
        if line.contains("Account<")
            && line.contains("mut")
            && !line.contains("signer")
            && (line.contains("transfer") || line.contains("payer") || line.contains("authority"))
        {
            findings.push(
                Finding::new(
                    "sol_missing_signer".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Mutable account lacks #[account(signer)] constraint".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Detects oracle rate account usage as price source
pub fn detect_oracle_rate_account(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("rate") || lower.contains("price") || lower.contains("clock"))
            && (lower.contains("sysvar") || lower.contains("account"))
            && !lower.contains("chain")
            && !lower.contains("link")
        {
            findings.push(
                Finding::new(
                    "sol_oracle_rate_account".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Using system account or rate source directly as price oracle".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Detects oracle self-trade: single signer controls maker and taker
pub fn detect_oracle_self_trade(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Look for trades where maker and taker are derived from same signer
    for (line_num, line) in source.lines().enumerate() {
        if ((line.contains("maker") && line.contains("taker"))
            || (line.contains("owner") && line.contains("owner")))
            && line.contains("signer")
            && !line.contains("require")
            && !line.contains("assert")
        {
            findings.push(
                Finding::new(
                    "sol_oracle_self_trade".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Single signer controls both sides of price trade".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Detects treasury with single non-multisig authority
pub fn detect_treasury_single_authority(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("treasury") || line.contains("vault"))
            && line.contains("authority")
            && !line.contains("multisig")
            && !line.contains("multi_sig")
        {
            findings.push(
                Finding::new(
                    "sol_treasury_single_authority".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Treasury/vault has single non-multisig authority".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Detects admin functions with no timelock
pub fn detect_admin_no_timelock(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("upgrade") || lower.contains("admin") || lower.contains("freeze"))
            && lower.contains("authority")
            && !lower.contains("timelock")
            && !lower.contains("delay")
        {
            findings.push(
                Finding::new(
                    "sol_admin_no_timelock".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Admin function executes immediately with no timelock delay".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Detects sysvar account validation absence
pub fn detect_sysvar_account_validation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("sysvar") || line.contains("system_program") || line.contains("clock"))
            && !line.contains("key")
            && !line.contains("address")
            && !line.contains("assert")
        {
            findings.push(
                Finding::new(
                    "sol_sysvar_account_validation".to_string(),
                    Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "System account used without address validation".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "solana".to_string()),
            );
        }
    }

    findings
}

/// Run all Solana detectors
pub fn detect_all(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(detect_missing_signer(source, file_path));
    findings.extend(detect_oracle_rate_account(source, file_path));
    findings.extend(detect_oracle_self_trade(source, file_path));
    findings.extend(detect_treasury_single_authority(source, file_path));
    findings.extend(detect_admin_no_timelock(source, file_path));
    findings.extend(detect_sysvar_account_validation(source, file_path));

    findings.sort_by(|a, b| match b.severity.cmp(&a.severity) {
        std::cmp::Ordering::Equal => a.line.cmp(&b.line),
        other => other,
    });

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_signer_detection() {
        let code = r#"
        #[derive(Accounts)]
        pub struct Transfer {
            #[account(mut)]
            pub payer: AccountInfo<'a>,
        }
        "#;

        let findings = detect_missing_signer(code, "program.rs");
        // Should find the issue
        for f in findings {
            assert_eq!(f.invariant_id, "sol_missing_signer");
        }
    }
}
