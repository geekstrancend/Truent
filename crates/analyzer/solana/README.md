# truent-analyzer-solana

Solana (Anchor/native Rust) analyzer for the Truent framework.

## Usage

```toml
[dependencies]
truent-analyzer-solana = "0.3.0"
truent-core = "0.3.0"
truent-ir = "0.3.0"
```

## Key Components

- `SolanaAnalyzer` — implements `ChainAnalyzer` (`analyze(&self, path: &Path) -> Result<ProgramModel>`)
- `anchor_parser::parse_anchor_accounts` — a real parser (not regex) for `#[derive(Accounts)]` structs; understands `Signer<'info>`, `has_one`/`owner`/`address` constraints, and `/// CHECK:` comments
- `detectors::run_all_detectors(source, file_path)` — the entry point the CLI calls; runs all 11 Solana pattern detectors plus the shared cross-chain `unauthorized_privileged_mutation` rule
- `semantic_model::build_semantic_model` — builds the chain-agnostic `truent_ir::SemanticModel` that rule consumes, reading guards off the real parsed `AccountSecurity` for each field (not re-derived from source text)

## Example

```rust
use truent_analyzer_solana::detectors::run_all_detectors;

let source = std::fs::read_to_string("lib.rs")?;
let findings = run_all_detectors(&source, "lib.rs");
println!("Found {} findings", findings.len());
```

## Detectors (11)

Missing signer checks, oracle rate/self-trade manipulation, treasury single
authority, admin without timelock, sysvar account validation, durable
nonce validation, rent exemption, PDA authority validation, unchecked
token/mint account type (fake-account substitution), fake sysvar
instructions account (unchecked `load_instruction_at`).

## License

MIT
