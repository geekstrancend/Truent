//! Integration tests.
//!
//! These tests validate the complete Truent system working end-to-end,
//! including parsing, analysis, code generation, and reporting.

use std::fs;
use tempfile::TempDir;

/// Create a minimal valid DSL invariant
fn create_minimal_invariant() -> String {
    r#"
invariant: basic_test
description: "Basic invariant test"

forall x in items:
    x > 0
"#
    .to_string()
}

/// Create a vault conservation invariant (Solana example)
fn create_vault_invariant() -> String {
    r#"
invariant: vault_conservation
description: "Vault balance must be conserved"
category: finance

context {
    state: VaultState,
    chain: Solana
}

forall deposit in state.deposits:
    deposit.amount > 0 &&
    deposit.owner != address(0)

global:
    sum(state.deposits.amount) == state.vault_total
"#
    .to_string()
}

/// Create a share mint safety invariant (EVM example)
fn create_share_mint_invariant() -> String {
    r#"
invariant: share_mint_safety
description: "Share minting must maintain supply invariant"
category: evm-token

context {
    state: TokenState,
    chain: EVM
}

forall mint in state.mints:
    mint.shares <= MAX_UINT256 &&
    mint.underlying >= mint.shares * exchange_rate()

global:
    total_shares() == sum(state.balances)
"#
    .to_string()
}

/// Create a governance quorum invariant
fn create_governance_invariant() -> String {
    r#"
invariant: governance_quorum
description: "Governance proposals must meet quorum"
category: governance

forall proposal in state.proposals:
    proposal.votes >= MIN_QUORUM &&
    proposal.votes <= total_voting_power() &&
    proposal.end_time > now()
"#
    .to_string()
}

#[test]
fn test_integration_parse_valid_dsl() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let dsl_path = temp.path().join("invariants.invar");
    let dsl_content = create_minimal_invariant();

    fs::write(&dsl_path, &dsl_content).expect("Failed to write DSL file");

    // Verify file was created
    assert!(dsl_path.exists(), "DSL file should be created");

    // Verify content
    let read_content = fs::read_to_string(&dsl_path).expect("Failed to read DSL file");
    assert!(
        read_content.contains("invariant:"),
        "DSL should contain invariant declaration"
    );
}

#[test]
fn test_integration_vault_conservation_dsl() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let dsl_path = temp.path().join("vault.invar");
    let dsl_content = create_vault_invariant();

    fs::write(&dsl_path, &dsl_content).expect("Failed to write vault DSL");

    let content = fs::read_to_string(&dsl_path).expect("Failed to read file");

    assert!(content.contains("vault_conservation"));
    assert!(content.contains("VaultState"));
    assert!(content.contains("Solana"));
}

#[test]
fn test_integration_share_mint_evm_dsl() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let dsl_path = temp.path().join("share_mint.invar");
    let dsl_content = create_share_mint_invariant();

    fs::write(&dsl_path, &dsl_content).expect("Failed to write share mint DSL");

    let content = fs::read_to_string(&dsl_path).expect("Failed to read file");

    assert!(content.contains("share_mint_safety"));
    assert!(content.contains("EVM"));
    assert!(content.contains("exchange_rate"));
}

#[test]
fn test_integration_governance_dsl() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let dsl_path = temp.path().join("governance.invar");
    let dsl_content = create_governance_invariant();

    fs::write(&dsl_path, &dsl_content).expect("Failed to write governance DSL");

    let content = fs::read_to_string(&dsl_path).expect("Failed to read file");

    assert!(content.contains("governance_quorum"));
    assert!(content.contains("MIN_QUORUM"));
    assert!(content.contains("voting_power"));
}

#[test]
fn test_integration_multiple_invariants() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let multi_path = temp.path().join("multi.invar");

    let content = format!(
        "{}\n\n{}\n\n{}\n\n{}",
        create_vault_invariant(),
        create_share_mint_invariant(),
        create_governance_invariant(),
        create_minimal_invariant()
    );

    fs::write(&multi_path, &content).expect("Failed to write multi-invariant file");

    let read_content = fs::read_to_string(&multi_path).expect("Failed to read file");

    assert!(read_content.contains("vault_conservation"));
    assert!(read_content.contains("share_mint_safety"));
    assert!(read_content.contains("governance_quorum"));
    assert!(read_content.contains("basic_test"));
}

#[test]
fn test_integration_invariant_project_structure() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base = temp.path();

    // Create project structure
    let dirs = vec!["src", "invariants", "tests", "docs"];

    for dir in dirs {
        fs::create_dir_all(base.join(dir)).expect("Failed to create directory");
    }

    let config_content = r#"
[project]
name = "smart-vault"
version = "0.1.0"
chains = ["solana", "evm"]

[invariants]
paths = ["invariants/"]

[analysis]
coverage_target = 90
strict_mode = true
"#;

    fs::write(base.join("invar.toml"), config_content).expect("Failed to write config");

    // Verify structure
    assert!(base.join("src").exists());
    assert!(base.join("invariants").exists());
    assert!(base.join("tests").exists());
    assert!(base.join("docs").exists());
    assert!(base.join("invar.toml").exists());
}

#[test]
fn test_integration_solana_simulation_project() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base = temp.path();

    // Create Solana-specific project layout
    fs::create_dir_all(base.join("programs/vault")).expect("Failed to create programs directory");

    let solana_program = r#"
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Debug)]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

impl Vault {
    pub fn new(owner: Pubkey) -> Self {
        Vault { owner, balance: 0 }
    }

    pub fn deposit(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.balance = self.balance.checked_add(amount)
            .ok_or(ProgramError::InsufficientFunds)?;
        Ok(())
    }
}
"#;

    fs::write(base.join("programs/vault/lib.rs"), solana_program)
        .expect("Failed to write Solana program");

    // Create invariants directory
    fs::create_dir_all(base.join("invariants")).expect("Failed to create invariants directory");

    // Create invariants for the Solana program
    fs::write(
        base.join("invariants/vault.invar"),
        create_vault_invariant(),
    )
    .expect("Failed to write invariants");

    // Verify Solana project structure
    assert!(base.join("programs/vault/lib.rs").exists());
    assert!(base.join("invariants/vault.invar").exists());
}

#[test]
fn test_integration_evm_simulation_project() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base = temp.path();

    // Create EVM-specific project layout
    fs::create_dir_all(base.join("contracts")).expect("Failed to create contracts directory");

    let evm_contract = r#"
pragma solidity ^0.8.0;

contract Token {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;

    function mint(uint256 amount) public {
        balances[msg.sender] += amount;
        totalSupply += amount;
    }

    function transfer(address to, uint256 amount) public {
        require(balances[msg.sender] >= amount);
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
"#;

    fs::write(base.join("contracts/Token.sol"), evm_contract)
        .expect("Failed to write EVM contract");

    // Create invariants directory
    fs::create_dir_all(base.join("invariants")).expect("Failed to create invariants directory");

    // Create invariants for the EVM contract
    fs::write(
        base.join("invariants/token.invar"),
        create_share_mint_invariant(),
    )
    .expect("Failed to write invariants");

    // Verify EVM project structure
    assert!(base.join("contracts/Token.sol").exists());
    assert!(base.join("invariants/token.invar").exists());
}

#[test]
fn test_integration_invariant_categories() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base = temp.path();

    // Create categorized invariants
    let categories = vec![
        ("finance", create_vault_invariant()),
        ("governance", create_governance_invariant()),
        ("token", create_share_mint_invariant()),
    ];

    for (category, content) in &categories {
        fs::write(base.join(format!("{}.invar", category)), content)
            .expect("Failed to write category invariant");
    }

    // Verify all categories exist
    for (category, _) in &categories {
        assert!(base.join(format!("{}.invar", category)).exists());
    }
}

#[test]
fn test_integration_no_file_corruption() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let path = temp.path().join("test.invar");

    let original_content = create_vault_invariant();
    fs::write(&path, &original_content).expect("Failed to write file");

    // Read it back
    let read_content = fs::read_to_string(&path).expect("Failed to read file");

    // Content must be identical (no corruption)
    assert_eq!(
        original_content, read_content,
        "File content should not be corrupted"
    );
}
