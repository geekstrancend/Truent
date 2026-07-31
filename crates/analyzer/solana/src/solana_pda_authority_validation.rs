/// Solana PDA Authority Validation Detector
///
/// Detects H53 vulnerability: PDA authority not properly validated
///
/// The vulnerability occurs when:
/// 1. PDA used without checking signer status
/// 2. Authority parameter not matched to PDA
/// 3. Attacker can provide different authority
/// 4. Can lead to unauthorized state changes
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref PDA_PATTERN: Regex =
        Regex::new(r"(?i)find_program_address|create_program_address|Pubkey::find_program_address")
            .unwrap();
    static ref AUTHORITY_PARAM: Regex =
        Regex::new(r"(?i)authority|admin|owner|signer|authorized").unwrap();
    static ref SIGNER_CHECK: Regex =
        Regex::new(r"(?i)require.*is_signer|require.*is_executable|account_info.*signer").unwrap();
    static ref AUTHORITY_VALIDATION: Regex = Regex::new(
        r"(?i)require.*authority.*==|require.*auth.*==|require.*signer.*==|require.*\.key\(\).*=="
    )
    .unwrap();
}

pub fn detect_solana_pda_authority_validation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !PDA_PATTERN.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 150, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        let has_authority_param = AUTHORITY_PARAM.is_match(&function_body);
        let has_validation = AUTHORITY_VALIDATION.is_match(&function_body);
        let _has_signer_check = SIGNER_CHECK.is_match(&function_body);

        if has_authority_param && !has_validation {
            findings.push(
                Finding::new(
                    "sol_pda_authority_validation".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "PDA used without validating authority parameter. Require authority == expected_pda.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H53".to_string())
                .with_metadata("exploit_name".to_string(), "Solana PDA Authority".to_string())
                .with_metadata("loss".to_string(), "$3.5M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "authority_validation".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add require!(authority.key() == expected_pda)".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unvalidated_authority() {
        let vulnerable = r#"
        let (pda, bump) = Pubkey::find_program_address(&[b"authority"], program_id);
        
        fn process_transaction(authority: AccountInfo) {
            // No validation of authority!
            process_operation(&authority);
        }
        "#;
        let findings = detect_solana_pda_authority_validation(vulnerable, "test.rs");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_with_authority_check() {
        let safe = r#"
        let (pda, bump) = Pubkey::find_program_address(&[b"authority"], program_id);
        
        fn process_transaction(authority: AccountInfo) {
            require!(authority.key() == &pda, "Invalid authority");
            process_operation(&authority);
        }
        "#;
        let findings = detect_solana_pda_authority_validation(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_signer_validation() {
        let safe = r#"
        let (pda, bump) = Pubkey::find_program_address(&[b"admin"], program_id);
        
        fn withdraw(payer: AccountInfo, admin: AccountInfo) {
            require!(admin.is_signer, "Admin not signer");
            require!(admin.key() == &pda, "Wrong admin");
        }
        "#;
        let findings = detect_solana_pda_authority_validation(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_pda_constraint_macro() {
        let safe = r#"
        #[account(
            mut,
            seeds = [b"authority"],
            bump,
            constraint = authority.key() == expected_authority @ CustomError::InvalidAuthority
        )]
        pub authority: AccountInfo<'info>,
        "#;
        let findings = detect_solana_pda_authority_validation(safe, "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_cpi_call_with_pda() {
        let safe = r#"
        let (pda, bump) = Pubkey::find_program_address(&[b"signer"], program_id);
        
        require!(authority.key() == &pda, "Not authorized");
        
        invoke_signed(
            &instruction,
            &[authority.clone()],
            &[&[b"signer", &[bump]]],
        )?;
        "#;
        let findings = detect_solana_pda_authority_validation(safe, "test.rs");
        assert!(findings.is_empty());
    }
}
