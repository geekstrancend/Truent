use truent_analyzer_solana::{parse_anchor_accounts, AccountSecurity, AnchorConstraint};

#[test]
fn test_signer_type_not_flagged() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeArena<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_arena(ctx: Context<InitializeArena>) -> Result<()> {
    Ok(())
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    assert_eq!(result.len(), 1);

    let init_struct = &result[0];
    assert_eq!(init_struct.name, "InitializeArena");
    assert_eq!(init_struct.fields.len(), 2);

    // Find the authority field
    let authority = init_struct
        .fields
        .iter()
        .find(|f| f.name == "authority")
        .unwrap();

    // Signer<'info> must NOT produce violations
    assert_eq!(authority.security, AccountSecurity::AnchorSigner);
    assert_eq!(
        authority.security.violation_severity(),
        None,
        "Signer should not be flagged"
    );
}

#[test]
fn test_pda_with_seeds_not_flagged() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        mut,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    assert_eq!(result.len(), 1);

    let vault_field = result[0].fields.iter().find(|f| f.name == "vault").unwrap();

    // Seeds constraint must suppress account validation violation
    match &vault_field.security {
        AccountSecurity::ConstrainedUnchecked { constraints } => {
            assert!(
                constraints
                    .iter()
                    .any(|c| matches!(c, AnchorConstraint::Seeds)),
                "Should have Seeds constraint"
            );
            // Adequately constrained accounts should not produce violations
            assert_eq!(
                vault_field.security.violation_severity(),
                None,
                "PDA with seeds should not be flagged"
            );
        }
        other => panic!("Expected ConstrainedUnchecked, got {:?}", other),
    }
}

#[test]
fn test_account_type_not_flagged() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    assert_eq!(result.len(), 1);

    let from_field = result[0].fields.iter().find(|f| f.name == "from").unwrap();

    // Account<'info, T> is framework-validated
    match &from_field.security {
        AccountSecurity::AnchorTypedAccount { type_name: _ } => {
            assert!(
                from_field.security.is_framework_validated(),
                "Account<T> should be framework validated"
            );
            assert_eq!(
                from_field.security.violation_severity(),
                None,
                "Account<T> should not be flagged"
            );
        }
        other => panic!("Expected AnchorTypedAccount, got {:?}", other),
    }
}

#[test]
fn test_check_comment_suppresses_violation() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ExecuteWithOracle<'info> {
    /// CHECK: This is the VRF oracle queue account from Switchboard.
    /// Its address is validated against known oracle program.
    pub oracle_queue: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    assert_eq!(result.len(), 1);

    let oracle_field = result[0]
        .fields
        .iter()
        .find(|f| f.name == "oracle_queue")
        .unwrap();

    // CHECK: comment must suppress critical violation
    match &oracle_field.security {
        AccountSecurity::CheckedByDeveloper { reason: _ } => {
            assert!(oracle_field.has_check_comment, "Should have CHECK comment");
            assert_eq!(
                oracle_field.security.violation_severity(),
                None,
                "Checked account should not be flagged"
            );
        }
        other => panic!("Expected CheckedByDeveloper, got {:?}", other),
    }
}

#[test]
fn test_truly_unchecked_account_flagged() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DangerousInstruction<'info> {
    pub mystery_account: AccountInfo<'info>,
    pub authority: Signer<'info>,
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    assert_eq!(result.len(), 1);

    let mystery_field = result[0]
        .fields
        .iter()
        .find(|f| f.name == "mystery_account")
        .unwrap();

    // This MUST produce a violation — it is genuinely unchecked
    assert_eq!(mystery_field.security, AccountSecurity::TrulyUnchecked);
    assert_eq!(
        mystery_field.security.violation_severity(),
        Some("critical"),
        "Unchecked without constraints should be critical"
    );
}

#[test]
fn test_program_type_not_flagged() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ExecuteInstruction<'info> {
    pub custom_program: Program<'info, MyCustomProgram>,
    pub authority: Signer<'info>,
}

pub struct MyCustomProgram;
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();

    // Find the ExecuteInstruction struct
    let exec_struct = result
        .iter()
        .find(|s| s.name == "ExecuteInstruction")
        .unwrap();
    let program_field = exec_struct
        .fields
        .iter()
        .find(|f| f.name == "custom_program")
        .unwrap();

    // Program<'info, T> is framework-validated
    match &program_field.security {
        AccountSecurity::AnchorProgram { program_name: _ } => {
            assert!(
                program_field.security.is_framework_validated(),
                "Program<T> should be framework validated"
            );
            assert_eq!(
                program_field.security.violation_severity(),
                None,
                "Program<T> should not be flagged"
            );
        }
        other => panic!("Expected AnchorProgram, got {:?}", other),
    }
}

#[test]
fn test_owner_constraint_suppresses_violation() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ValidateAccount<'info> {
    #[account(owner = system_program.key())]
    pub account_data: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    let account_field = result[0]
        .fields
        .iter()
        .find(|f| f.name == "account_data")
        .unwrap();

    // Owner constraint must suppress violation
    match &account_field.security {
        AccountSecurity::ConstrainedUnchecked { constraints } => {
            assert!(
                constraints
                    .iter()
                    .any(|c| matches!(c, AnchorConstraint::Owner(_))),
                "Should have Owner constraint"
            );
            // Accounts with owner checks are adequately constrained
            assert_eq!(
                account_field.security.violation_severity(),
                None,
                "Account with owner constraint should not be flagged"
            );
        }
        other => panic!("Expected ConstrainedUnchecked with owner, got {:?}", other),
    }
}

#[test]
fn test_address_constraint_suppresses_violation() {
    let source = r#"
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ValidateAddress<'info> {
    #[account(address = KNOWN_AUTHORITY)]
    pub known_account: AccountInfo<'info>,
}

pub const KNOWN_AUTHORITY: Pubkey = solana_program::declare_id!("11111111111111111111111111111111");
    "#;

    let result = parse_anchor_accounts(source, "test.rs").unwrap();
    let known_field = result[0]
        .fields
        .iter()
        .find(|f| f.name == "known_account")
        .unwrap();

    // Address constraint must suppress violation
    match &known_field.security {
        AccountSecurity::ConstrainedUnchecked { constraints } => {
            assert!(
                constraints
                    .iter()
                    .any(|c| matches!(c, AnchorConstraint::Address(_))),
                "Should have Address constraint"
            );
            assert_eq!(
                known_field.security.violation_severity(),
                None,
                "Account with address constraint should not be flagged"
            );
        }
        other => panic!(
            "Expected ConstrainedUnchecked with address, got {:?}",
            other
        ),
    }
}
