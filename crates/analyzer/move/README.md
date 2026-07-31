# truent-analyzer-move

Move language analyzer for the Truent framework (Aptos, Sui, and other
Move-based networks).

Performs static analysis on Move module *source* (not compiled bytecode) to
detect security invariant violations and unsafe patterns.

## Usage

```toml
[dependencies]
truent-analyzer-move = "0.3.0"
truent-core = "0.3.0"
truent-ir = "0.3.0"
```

## Key Components

- `MoveAnalyzer` — implements the `ChainAnalyzer` trait (`analyze(&self, path: &Path) -> Result<ProgramModel>`), the same interface every chain analyzer in the workspace shares
- `run_all_detectors(source, file_path)` — the entry point the CLI actually calls; runs all 7 pattern detectors plus the shared cross-chain `unauthorized_privileged_mutation` rule
- `build_semantic_model(source, file_path)` — builds the chain-agnostic `truent_ir::SemanticModel` consumed by that shared rule
- `tree_sitter_grammar` — a vendored Sui Move tree-sitter grammar (see `vendor/tree-sitter-move-sui/PROVENANCE.md`) backs `build_semantic_model`'s real AST parsing; falls back to a regex heuristic if a file doesn't parse cleanly (the upstream grammar is itself still work-in-progress)

## Example

```rust
use truent_analyzer_move::run_all_detectors;

let source = std::fs::read_to_string("Vault.move")?;
let findings = run_all_detectors(&source, "Vault.move");
println!("Found {} findings", findings.len());
```

## Detectors (7)

- Resource destruction (`move_resource_destruction`)
- Type safety violations (`move_type_safety_violation`)
- Access control missing (`move_access_control_missing`)
- Manual overflow/bitmask-check patterns (`move_manual_overflow_check`)
- Liquidity conservation, admin timelock, oracle spot price (`detectors.rs`)

Plus the shared `unauthorized_privileged_mutation` rule (flags privileged
mutations — fund transfers, authority changes, upgrades, account closes —
with no capability/authorization check reaching them), which every chain
analyzer in the workspace contributes to and shares verbatim.

## Supported Platforms

- Sui (primary target for the vendored grammar)
- Aptos
- Other Move-compatible networks (best-effort; falls back to the regex
  heuristic for syntax the grammar doesn't recognize)

## License

MIT
