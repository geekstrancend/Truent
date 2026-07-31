/// Solana Account Rent Exemption Detector
///
/// Detects H55 vulnerability: Accounts not properly rent-exempt
///
/// The vulnerability occurs when:
/// 1. Mutable account lacks rent exemption requirement
/// 2. Account can be reclaimed by Solana runtime if rent unpaid
/// 3. State loss or double-spend possible
/// 4. PDA accounts must be properly initialized
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref ACCOUNT_MUT: Regex =
        Regex::new(r"(?i)account_info.*is_signer\s*=|#\[account\(mut\)\]|AccountInfo.*mut")
            .unwrap();
    static ref RENT_EXEMPT_CHECK: Regex =
        Regex::new(r"(?i)rent\.is_exempt|lamports.*?\>=.*?rent|rent_exempt").unwrap();
    static ref TRANSFER_TO_ACCOUNT: Regex =
        Regex::new(r"(?i)transfer_with_seed|rent\.minimum_balance").unwrap();
}

pub fn detect_solana_rent_exemption(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !ACCOUNT_MUT.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 100, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        let has_rent_check = RENT_EXEMPT_CHECK.is_match(&function_body);

        if !has_rent_check {
            findings.push(
                Finding::new(
                    "sol_rent_exemption_check".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Mutable account lacks rent exemption validation. Verify: rent.is_exempt(lamports, account_size)".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H55".to_string())
                .with_metadata("exploit_name".to_string(), "Solana Rent Exemption".to_string())
                .with_metadata("loss".to_string(), "$6.7M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "rent_violation".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add rent.is_exempt(account.lamports(), account.data.len())".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_rent_check() {
        let vulnerable = r#"
        #[account(mut)]
        pub user_account: AccountInfo<'info>,
        "#;
        let findings = detect_solana_rent_exemption(vulnerable, "test.rs");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_with_rent_exempt_check() {
        let safe = r#"
        #[account(mut)]
        pub user_account: AccountInfo<'info>,
        
        let is_exempt = rent.is_exempt(user_account.lamports(), user_account.data.len());
        require!(is_exempt, "Not rent exempt");
        "#;
        let findings = detect_solana_rent_exemption(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_pda_creation() {
        let safe = r#"
        let required_lamports = rent.minimum_balance(account_size);
        system_program::create_account(
            CpiContext::new(...),
            required_lamports,
            account_size,
            &bumps.pda,
        )?;
        "#;
        let findings = detect_solana_rent_exemption(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_transfer_with_seed() {
        let safe = r#"
        let rent_exempt = rent.minimum_balance(MY_ACCOUNT_SIZE);
        system_program::transfer_with_seed(
            CpiContext::new(...),
            rent_exempt,
            MY_ACCOUNT_SIZE,
            &seed,
            &bump,
        )?;
        "#;
        let findings = detect_solana_rent_exemption(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_immutable_account() {
        let ignored = r#"
        pub account: AccountInfo<'info>,
        "#;
        let findings = detect_solana_rent_exemption(ignored, "test.rs");
        assert!(findings.is_empty()); // Immutable accounts don't need rent check
    }
}
