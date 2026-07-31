# truent-dynamic-core

Chain-agnostic dynamic/coverage-guided invariant fuzzing engine.

This crate contains no VM. It only knows how to:

- generate random call sequences against a described function surface (`FunctionSpec`), biased toward edge values (0, 1, `u256::MAX`, …) the way property fuzzers like Foundry/Echidna do,
- drive an [`ExecutionBackend`](src/backend.rs) (deploy/call/snapshot/revert — implemented once per chain),
- check [`Invariant`](src/invariant.rs)s after every call (`MonotonicInvariant`, `ConservationInvariant`),
- and shrink a failing sequence to a minimal reproduction via delta-debugging.

Chain-specific execution lives in its own crate — see `truent-dynamic-evm` for the `revm`-backed implementation for real Solidity bytecode. This crate's own test suite proves the engine logic (generation, execution, shrinking, invariant checking) correct against an in-memory mock, so it never depends on a real VM to be verified.

## Adding a new invariant

Implement `Invariant` (`name`, `check`, optionally `reset` for anything that remembers state across calls) — see `MonotonicInvariant` for a stateful example and `ConservationInvariant` for a stateless one.

## Adding a new chain backend

Implement `ExecutionBackend` (`call`, `snapshot`, `revert_to`) against that chain's execution environment. `truent-dynamic-evm`'s `RevmBackend` is the reference implementation.
