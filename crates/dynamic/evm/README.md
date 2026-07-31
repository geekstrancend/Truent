# truent-dynamic-evm

`revm`-backed dynamic invariant fuzzing for EVM/Solidity contracts, implementing `truent-dynamic-core`'s `ExecutionBackend` for real bytecode.

## Feature flags

- `revm-backend` (not on by default): pulls in `revm` and compiles `backend::RevmBackend`, the real execution engine. Kept optional so plain `cargo check`/`cargo metadata` (what IDE tooling calls constantly) don't require fetching `revm` just to typecheck the ABI/bytecode parsing bridge in `solc_bridge.rs`. The CLI enables this feature explicitly.

## Modules

- `types.rs` — `CompiledContract` (init code + parsed function surface), dependency-free.
- `solc_bridge.rs` — parses `truent_utils::SolcManager`'s `--combined-json abi,bin` output into `truent_dynamic_core::FunctionSpec`s and raw creation bytecode. Computes selectors via keccak256, matching Solidity's `name(type1,type2)` signature hashing.
- `backend.rs` (behind `revm-backend`) — deploys and calls into contracts via `revm`, with DB-clone-based snapshot/revert for the shrinker.
- `lib.rs` — `auto_detect_invariants` (ERC20-shaped conservation, monotonic accumulator getters) and `fuzz_solidity_source` (compile via solc, auto-detect invariants, run the fuzzer).

## Usage

```rust,ignore
use truent_dynamic_core::FuzzConfig;

let config = FuzzConfig {
    seed: 0,
    max_runs: 500,
    sequence_depth: 12,
    actors: vec![[1u8; 20], [2u8; 20], [3u8; 20]],
};

if let Some(violation) = truent_dynamic_evm::fuzz_solidity_source(source, config)? {
    println!("{}", truent_dynamic_core::format_poc(&violation));
}
```

Or via the CLI: `truent fuzz --dynamic --chain evm path/to/Contract.sol`.
