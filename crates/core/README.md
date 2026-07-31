# truent-core

Core types, traits, and shared utilities for the Truent framework. Every
other crate in the workspace depends on this one.

## Usage

```toml
[dependencies]
truent-core = "0.3.0"
```

## Key Types

- `Finding` — a single detected violation (`invariant_id`, `severity`, `file`, `line`, `col`, `message`, `snippet`, plus optional metadata/source-fragment via the builder methods)
- `Severity` — `Critical | High | Medium | Low | Info`
- `Invariant`, `ProgramModel`, `FunctionModel`, `StateVar` — the program model chain analyzers produce (also re-exported from `truent-ir`)
- `ChainAnalyzer`, `CodeGenerator`, `Simulator` — the traits each chain's analyzer/generator crate implements
- `Config`, `ChainConfig`, `AlertConfig`, `InvariantConfig` — project configuration (`.truent.toml`)
- `CodeFuzzer`, `FuzzResult` — deterministic, seeded synthetic-pattern generation used by the fuzz-testing infrastructure

## Example

```rust
use truent_core::{Finding, Severity};

let finding = Finding::new(
    "evm_reentrancy_classic".to_string(),
    Severity::Critical,
    "Vault.sol".to_string(),
    42,
    0,
    "External call before state update".to_string(),
    "(bool ok,) = msg.sender.call{value: amount}(\"\");".to_string(),
)
.with_metadata("detector".to_string(), "pattern_match".to_string());
```

## License

MIT
