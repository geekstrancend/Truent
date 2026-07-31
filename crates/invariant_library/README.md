# truent-library

Standard library of built-in invariants for the Truent framework.

Ships 22 pre-configured security checks: 10 for EVM, 7 for Solana, 5 for
Move. Each compiles through the real DSL parser into an executable
`Expression` (not a placeholder) — see `crates/dsl_parser`.

## Usage

```toml
[dependencies]
truent-library = "0.3.0"
truent-core = "0.3.0"
```

## Loading invariants

```rust
use truent_library::InvariantLibrary;

// Built-in defaults for one chain:
let lib = InvariantLibrary::with_defaults("evm");
println!("EVM has {} checks", lib.count());

for invariant in lib.all() {
    println!("{}: {}", invariant.name, invariant.expression);
}

// Or start empty and add your own:
let mut custom = InvariantLibrary::new();
custom.add("custom".to_string(), my_invariant);
```

There's also a TOML-based loader (`loader.rs`) that parses invariant
definitions from a table (see `examples/invariants.toml`) through the same
DSL parser, for invariants defined outside this crate's Rust source.

## Available Invariants

**EVM** (10): `evm_reentrancy_protection`, `evm_integer_overflow`,
`evm_integer_underflow`, `evm_unchecked_returns`,
`evm_delegatecall_injection`, `evm_access_control`,
`evm_timestamp_dependence`, `evm_frontrunning`,
`evm_uninitialized_pointers`, `evm_division_by_zero`

**Solana** (7): `sol_signer_checks`, `sol_account_validation`,
`sol_integer_overflow`, `sol_rent_exemption`, `sol_pda_derivation`,
`sol_lamport_balance`, `sol_instruction_parsing`

**Move** (5): `move_access_control`, `move_integer_overflow`,
`move_resource_leaks`, `move_type_mismatches`, `move_signer_requirements`

These are IDs handled by the DSL/invariant layer specifically; they're
distinct from (and smaller in number than) the 50 pattern-based detectors
in `truent-analyzer-{evm,solana,move}`, which run directly against source
text/AST rather than through the DSL.

## License

MIT
