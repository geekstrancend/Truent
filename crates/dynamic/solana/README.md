# truent-dynamic-solana

Dynamic invariant fuzzing for Solana programs — the account-model counterpart
to `truent-dynamic-evm`.

Solana's execution model is account-based (an instruction is a program id, a
list of `AccountMeta`s, and an opaque data blob), not the EVM's flat-calldata,
single-caller shape. So this crate defines its own call model, invariant
oracles, instruction generator, and fuzz/shrink loop rather than forcing the
EVM types onto it. Everything else — the search + delta-debug shrink to a
minimal proof-of-concept — mirrors the proven EVM engine.

## What it checks

- **Token conservation** — `sum(token account amounts) == mint supply`. Catches
  "mint out of thin air": an instruction that credits a balance without
  updating supply.
- **Account-owner integrity** — an account's `owner` must not change (or must
  change only as declared). Catches unchecked ownership reassignment.

Both come with the minimal reproducing instruction sequence.

## Driving it from the CLI

```bash
truent fuzz path/to/idl.json --dynamic --chain solana --plan plan.json
```

Two inputs, because one is not enough:

- **The Anchor IDL** gives the instruction surface — discriminators, argument
  types, and each account's signer/writable role. Both IDL layouts are
  supported: 0.30+ (explicit `discriminator`, `signer`/`writable`) and legacy
  (no discriminator — recomputed as `sha256("global:<snake_case>")[..8]` — with
  `isSigner`/`isMut`).
- **The fuzz plan** (`--plan`) gives what an IDL cannot: the genesis accounts,
  the account pool, which positions are *pinned*, and the invariants to check.
  A plan with no invariants is rejected rather than silently reporting a clean
  run.

Instructions taking non-fixed-width arguments (`String`, `Vec`, structs) can't
be encoded correctly, so they're excluded and **reported** — partial coverage
never reads as full coverage.

### Pinning

An IDL names each account position (`mint`, `vault`, `authority`). Pin one and
that position always resolves to that account:

```json
"pin": { "mint": "<mint pubkey>" }
```

This matters for correctness, not just speed. Unpinned, the generator can draw
a token account for the `mint` slot, and a *correct* program will then appear
to break conservation — a false positive rather than a finding.

## Backends

- **Default (no features):** the engine is proven against an in-memory
  `MockSvm` (a tiny SPL-token-like program with a deliberately buggy airdrop),
  exactly as the EVM crate proved its engine before wiring revm. Builds and
  tests with no Solana dependency.
- **`litesvm-backend`:** a real in-process SVM (`litesvm`) that runs compiled
  BPF bytecode. Pins the same granular `solana-*` 2.2 crates litesvm uses so
  the whole thing links against one coherent Solana version. Seed a genesis,
  deploy the program `.so`, and the same invariant search runs over real code.

With `litesvm-backend` enabled, the test suite additionally runs the fuzzer
against the **real SPL Token program** that litesvm embeds: it asserts a
`MintTo` genuinely executes and moves supply (so the run isn't vacuous), and
that fuzzing audited token bytecode reports **no** violation. Catching is
proven against the mock's buggy airdrop; not-crying-wolf is proven against real
code.

```bash
cargo test  -p truent-dynamic-solana                             # engine proof (mock)
cargo test  -p truent-dynamic-solana --features litesvm-backend  # + real SPL Token e2e
```
