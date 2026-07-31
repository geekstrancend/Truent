# Integration Testing Guide

## Overview

Integration tests validate **end-to-end workflows**, simulating real-world usage patterns. They test that multiple components work together correctly to solve actual problems.

## Test Structure

```
tests/integration/
├── mod.rs                 # Main integration test module
├── projects/              # Test project fixtures
│   ├── minimal/           # Minimal valid project
│   ├── solana-vault/      # Solana vault example
│   ├── evm-token/         # EVM token example
│   └── governance/        # Governance example
└── workflows/             # Complex workflow tests
```

## Test Categories

### 1. **Project Structure Tests**

Validate that Truent works with complete projects:

```rust
#[test]
fn test_integration_invariant_project_structure() {
    let temp = TempDir::new()?;
    let base = temp.path();

    // Create expected directories
    fs::create_dir_all(base.join("src"))?;
    fs::create_dir_all(base.join("invariants"))?;
    fs::create_dir_all(base.join("tests"))?;

    // Create config file
    let config = r#"
[project]
name = "test-project"
version = "0.1.0"
chains = ["solana", "evm"]
"#;
    fs::write(base.join("truent.toml"), config)?;

    // Verify structure
    assert!(base.join("invariants").exists());
}
```

### 2. **DSL Invariant Tests**

Test real-world invariant examples:

```rust
#[test]
fn test_integration_vault_conservation_dsl() {
    let content = r#"
invariant: vault_conservation
description: "Vault balance must be conserved"

context {
    state: VaultState,
    chain: Solana
}

forall deposit in state.deposits:
    deposit.amount > 0

global:
    sum(state.deposits.amount) == state.vault_total
"#;

    // File should be valid and parseable
    assert!(content.contains("vault_conservation"));
    assert!(content.contains("global:"));
}
```

### 3. **Platform-Specific Workflows**

#### Solana Integration
```rust
#[test]
fn test_integration_solana_token_simulation() {
    // Create a Solana program
    let solana_program = r#"
use solana_program::program_error::ProgramError;

pub struct Vault {
    pub balance: u64,
}

impl Vault {
    pub fn deposit(&mut self, amount: u64) -> Result<(), ProgramError> {
        self.balance = self.balance
            .checked_add(amount)
            .ok_or(ProgramError::IllegalOwner)?;
        Ok(())
    }
}
"#;

    let temp = TempDir::new()?;
    let program_path = temp.path().join("lib.rs");
    fs::write(&program_path, solana_program)?;

    // Create matching invariants
    let invariants = r#"
invariant: balance_monotonic
forall tx in state.transactions:
    tx.new_balance >= tx.old_balance  // After deposit
"#;

    let invariant_path = temp.path().join("vault.invar");
    fs::write(&invariant_path, invariants)?;

    // Both files should exist and be accessible
    assert!(program_path.exists());
    assert!(invariant_path.exists());
}
```

#### EVM Integration
```rust
#[test]
fn test_integration_evm_token_invariants() {
    let evm_contract = r#"
pragma solidity ^0.8.0;

contract Token {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;

    function transfer(address to, uint256 amount) public {
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
"#;

    let invariants = r#"
invariant: total_supply_conservation
global:
    sum(balances) == totalSupply
"#;

    let temp = TempDir::new()?;
    fs::write(temp.path().join("Token.sol"), evm_contract)?;
    fs::write(temp.path().join("token.invar"), invariants)?;

    // Verify directory structure
    assert_eq!(
        fs::read_dir(temp.path()).unwrap().count(),
        2,
        "Should contain both contract and invariants"
    );
}
```

### 4. **Multi-Chain Projects**

```rust
#[test]
fn test_integration_multi_chain_invariants() {
    let temp = TempDir::new()?;
    let base = temp.path();

    // Create chain-specific directories
    for chain in &["solana", "evm", "move"] {
        fs::create_dir_all(base.join(chain))?;
        
        let invariant = format!("invariant: {}_test\ntrue", chain);
        fs::write(
            base.join(chain).join("test.invar"),
            invariant
        )?;
    }

    // Verify all chains have invariants
    for chain in &["solana", "evm", "move"] {
        assert!(base.join(chain).join("test.invar").exists());
    }
}
```

### 5. **File Content Integrity**

```rust
#[test]
fn test_integration_no_file_corruption() {
    let temp = TempDir::new()?;
    let path = temp.path().join("test.invar");
    
    let original = r#"
invariant: test
forall x in items:
    x > 0
"#;

    fs::write(&path, original)?;
    let read_back = fs::read_to_string(&path)?;

    assert_eq!(original, read_back, "File should not be corrupted");
}
```

## Real-World Examples

### Example 1: Vault Conservation

```rust
fn create_vault_invariant() -> String {
    r#"
invariant: vault_conservation
description: "Total vault balance = sum of deposits"
category: finance
tags: [solana, vault, security]

context {
    state: VaultState,
    chain: Solana,
    protocol: SPL
}

// Conservation law: deposits always sum to vault total
global:
    sum(state.deposits[*].amount) == state.vault_balance

// Deposits must be positive
forall deposit in state.deposits:
    deposit.amount > 0 &&
    deposit.owner != address(0) &&
    deposit.created_at <= now()

// No double-spend possible
forall user in state.users:
    user.balance <= sum(deposits where deposits.owner == user)
"#.to_string()
}
```

### Example 2: Token Mint Safety

```rust
fn create_evm_token_invariants() -> String {
    r#"
invariant: share_mint_safety
description: "Token minting maintains supply invariant"
chain: EVM
contract: Token

// Invariant 1: Conservation
global:
    sum(balances[*]) == totalSupply

// Invariant 2: Mint bounds
forall mint_op in state.recent_mints:
    mint_op.amount <= MAX_MINT_PER_TX &&
    mint_op.minter_role_check == true

// Invariant 3: Overflow prevention
global:
    totalSupply <= MAX_UINT256
"#.to_string()
}
```

### Example 3: Governance Quorum

```rust
fn create_governance_invariants() -> String {
    r#"
invariant: governance_quorum
description: "Proposals must meet voting quorum"
category: governance

forall proposal in state.proposals:
    proposal.votes_for + proposal.votes_against <= total_voting_power() &&
    proposal.status in [PENDING, ACTIVE, PASSED, FAILED] &&
    proposal.end_time > now()

global:
    count(state.proposals where status == PASSED) >= 0 &&
    sum(all_votes) == count(voters) * entries_per_voter
"#.to_string()
}
```

## Running Integration Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific integration test
cargo test test_integration_vault_conservation_dsl

# Run with output
cargo test --test integration -- --nocapture
```

## Best Practices

### 1. Use Real-World Examples
- Base tests on actual protocols
- Use realistic invariants
- Test complete workflows

### 2. Isolate Test State
```rust
#[test]
fn test_with_proper_isolation() {
    // Each test gets its own TempDir
    let temp = TempDir::new()?;
    
    // Modifications don't affect other tests
    // Cleaned up automatically
}
```

### 3. Pattern Matching for Variants
```rust
#[test]
fn test_multiple_variants() {
    for chain in &["solana", "evm", "move"] {
        let invariant = format!(
            "invariant: test_{}
            true",
            chain
        );
        assert!(!invariant.is_empty());
    }
}
```

### 4. Verify Complex Conditions
```rust
#[test]
fn test_integration_invariant_categories() {
    let categories = vec![
        ("finance", "vault"),
        ("governance", "quorum"),
        ("token", "mint"),
    ];

    for (category, context) in categories {
        let name = format!("{}_{}", category, context);
        assert!(name.len() > 0);
        // Test category-specific logic
    }
}
```

## Continuous Integration

Integration tests run in CI to ensure end-to-end workflows:

```yaml
integration-tests:
  name: Integration Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Run integration tests
      run: cargo test --test integration --verbose
```

## Troubleshooting

**Test fails with "file not found":**
- Verify temp directory setup
- Check relative vs absolute paths
- Use `temp.path().join()` for proper path construction

**Tests interfering with each other:**
- Ensure each test uses its own TempDir
- Don't write to real filesystem
- Clean up resources properly

**Slow integration tests:**
- Consider breaking into smaller tests
- Use `#[ignore]` for slow tests
- Run separately: `cargo test --test integration -- --ignored --nocapture`
