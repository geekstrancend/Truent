# truent-analyzer-soroban

Static analyzer for Soroban (Stellar) smart contracts: Rust source compiled
against `soroban-sdk`, using `#[contract]`/`#[contractimpl]` and `Env`-scoped
storage.

## Usage

```toml
[dependencies]
truent-analyzer-soroban = "0.3.0"
truent-core = "0.3.0"
```

```rust
use truent_analyzer_soroban::run_all_detectors;

let source = std::fs::read_to_string("contract.rs")?;
let findings = run_all_detectors(&source, "contract.rs");
for finding in findings {
    println!("[{:?}] {}: {}", finding.severity, finding.invariant_id, finding.message);
}
```

`SorobanAnalyzer` also implements `truent_core::traits::ChainAnalyzer` for
building a `ProgramModel` (`analyzer::SorobanAnalyzer.analyze(path)`).

## How it works

`syn` parses the source into a real AST; `soroban_parser::parse_contract_functions`
walks every `pub fn` inside a `#[contractimpl]` block and extracts, per
function, whether it calls `require_auth`, whether it's an initializer with
a re-entry guard, whether it does raw vs. `checked_*` arithmetic, whether it
writes `persistent`/`temporary` storage and extends TTL, and whether a
cross-contract call precedes a later storage write. Every detector in
`detectors.rs` reads off that shared fact set — no detector re-parses source
text on its own. `semantic_model.rs` feeds the same facts into the shared
chain-agnostic `truent_ir::rules::find_unauthorized_privileged_mutations`
rule (`require_auth` is Soroban's equivalent of Solana's `Signer<'info>` or
EVM's `onlyOwner`).

## Detectors

| ID | Severity | Checks |
|---|---|---|
| `sor_missing_require_auth` | Critical | A function whose name suggests a privileged action (withdraw/transfer/pay/mint/burn/set_admin/set_owner/remove/upgrade) has no `require_auth()` call anywhere in its body |
| `sor_unprotected_upgrade` | Critical | `update_current_contract_wasm` called with no `require_auth()` guarding it |
| `sor_reinitialization` | High | `initialize`/`init` has no storage `.has(` check before its first `.set(`, so it can be called more than once |
| `sor_unchecked_arithmetic` | High | Raw `+`/`-`/`*` with no `checked_add`/`checked_sub`/`checked_mul`/`checked_div` call in the same function — Rust release builds don't trap overflow by default |
| `sor_storage_ttl_not_extended` | Medium | A `persistent()` storage write with no `extend_ttl`/`.bump(` call anywhere in the function |
| `sor_temporary_storage_critical_state` | Medium | A `temporary()` write whose key/type name looks durable (balance/admin/owner/supply/price) — `temporary()` entries don't survive |
| `sor_reentrancy_external_call` | High | A cross-contract call (`invoke_contract`/`Client::new`) precedes a later storage write in the same function (checks-effects-interactions violation) |
| `sor_thin_liquidity_oracle_price` | High | A price read from a single spot-price call (`get_price`/`.price(`/`spot_price`/`exchange_rate`) with no TWAP/multi-source corroboration anywhere in the function |
| `sor_unhandled_panic` | Low | `.unwrap()`/`.expect(` calls, which abort the whole invocation |

Plus the shared `unauthorized_privileged_mutation` (Critical) rule from
`truent_ir::rules`.

## Known limitations

- Single-source-file scope: no cross-file storage-key-collision analysis, no
  WASM-level analysis, no resolution of constants/config defined in a
  sibling `Cargo.toml` (e.g. this crate can't check whether
  `overflow-checks = true` is set in the contract's release profile — see
  `sor_unchecked_arithmetic`'s message for that recommendation instead).
- Detection is source-text-pattern-based against re-stringified `syn` AST
  fragments (the same technique `truent-analyzer-solana` uses), not full
  data-flow analysis — e.g. `sor_unchecked_arithmetic` flags a function if
  it contains *any* raw arithmetic and *no* `checked_*` call anywhere in
  that function, not per-operation.

## License

MIT
