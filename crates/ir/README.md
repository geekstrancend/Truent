# truent-ir

Chain-agnostic intermediate representation for Truent.

Re-exports the core program/invariant model types from `truent-core`, and
defines the shared semantic IR that lets a detection rule be written once
and apply unmodified to every chain.

## Usage

```toml
[dependencies]
truent-ir = "0.3.0"
truent-core = "0.3.0"
```

## Key Types

Re-exported from `truent-core::model`:
- `Expression`, `BinaryOp`, `LogicalOp` — invariant expression AST
- `Invariant`, `ProgramModel`, `FunctionModel`, `StateVar` — the program model chain analyzers produce
- `GenerationOutput`, `SimulationReport`

Defined in this crate:
- `SemanticModel` — a chain-agnostic set of [`PrivilegedMutation`] sites extracted from one source file, independent of the chain's native syntax
- `PrivilegedMutation` — an entry point performing a sensitive state change (`MutationKind`: fund transfer, authority change, upgrade, account close), plus whatever `AuthorizationCheck`s guard it
- `AuthorizationCheck` / `AuthCheckKind` — a guard found protecting a mutation (signer, role/capability, multisig, or other)
- `AnalysisContext` — accumulates warnings/validity state during analysis
- `DependencyGraph` — dependency ordering for generation output

## Shared cross-chain rule

`truent_ir::rules::find_unauthorized_privileged_mutations` is the first rule
built on this pattern: it flags any `PrivilegedMutation` with no guard, and
is written once in `crates/ir/src/rules.rs`. Each chain analyzer (EVM,
Solana, Move) builds its own `SemanticModel` from its own native syntax —
EVM from a solc AST, Solana from a real Anchor-account parse, Move from a
vendored tree-sitter grammar — and the rule itself never changes to support
a new chain. This is the intended pattern for adding more chain-agnostic
rules: extend `SemanticModel` with whatever new fact the rule needs, then
have each analyzer populate it.

## Example

```rust
use truent_ir::{SemanticModel, PrivilegedMutation, MutationKind};
use truent_ir::rules::find_unauthorized_privileged_mutations;

let mut model = SemanticModel::new("evm", "Vault.sol");
model.mutations.push(PrivilegedMutation {
    entry_point: "withdraw".to_string(),
    kind: MutationKind::FundTransfer,
    line: 42,
    guards: vec![], // no authorization check found
});

let findings = find_unauthorized_privileged_mutations(&model);
assert_eq!(findings.len(), 1);
```

## License

MIT
