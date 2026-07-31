# Engine verification of LLM findings (Phase 3 — the differentiator)

A prompt-only auditor stops at "I think this is a bug." Truent does not: it
tries to **make the bug fire in a real EVM** and, if it does, ships the runnable
sequence. This file is how you turn a `REASONED` lead into a `VERIFIED` finding.

## What the engine can verify by execution today

Truent's dynamic fuzzer (`truent fuzz --dynamic`) deploys the contract in an
in-memory `revm`, drives adversarial call sequences, and after **every** call
checks a set of auto-detected invariants, shrinking any violation to a minimal
proof-of-concept. It automatically recognizes and checks these property shapes
from the contract's ABI — you do not have to write them:

| Property shape | Auto-detected from | What a violation proves |
|---|---|---|
| **Conservation** | `totalSupply()` + `balanceOf(address)` | value minted/credited with no matching supply change |
| **Monotonicity** | a no-arg accumulator getter (`totalSupply`, `totalAssets`, `exchangeRate`, `sharePrice`, `cumulative*`, `checkpoint*`) | a quantity that must never decrease went down |
| **Access control** | `owner()` + `transferOwnership(address)` | ownership changed via a call from a non-owner |
| **Reentrancy** | execution trace (call-stack inspector) | a contract re-entered itself and wrote state after the external call (CEI violation) |

So: **if a lead maps to one of these shapes, verification is just running the
fuzzer** — no harness authoring.

## The verification loop

For each `REASONED` candidate that names a property:

1. **Match it to a shape above.** "Airdrop credits balance without updating
   supply" → *conservation*. "Share price dropped after a sequence" →
   *monotonicity*. "Non-owner rotated the admin" → *access control*.
   "Withdraw re-enters before zeroing balance" → *reentrancy*.
2. **Run the fuzzer against the contract that exhibits the shape:**
   ```bash
   truent fuzz <file.sol> --dynamic --chain evm --iterations 1000 --seed 1
   ```
   The fuzzer auto-detects the shape and searches for a violating sequence.
3. **Read the result.** On a hit, Truent prints the violated invariant, the
   message, and a **minimal reproduction** — e.g.:
   ```
   Invariant violated: ERC20 conservation: sum(balanceOf) == totalSupply()
   sum(balanceOf) = 2000 != totalSupply() = 1000
   Reproduction (2 calls):
     1. airdrop(0x0101…, 1000)   [caller=0x1111…]
     2. …
   Failing step: #1 (airdrop)
   ```
   That block is the PoC. **Promote the finding to `VERIFIED`** and paste the
   reproduction into `proof:`.
4. **Vary the seed** (`--seed 2`, `--seed 3`) before concluding "not
   reproducible" — the search is seeded and deterministic per seed. If several
   seeds and a raised `--iterations` all hold the invariant, the engine is
   telling you the property is *not* violated: **downgrade or drop** the lead.
   Do not report a property the engine actively could not break as a finding.

## Bespoke properties (no matching shape)

If a lead's property doesn't map to an auto-detected shape (a protocol-specific
equivalence, a multi-contract flow), you cannot fully machine-verify it yet.
Then:

- **Reduce it** toward a shape the engine checks (e.g. reframe "vault share
  math is wrong" as a monotonic `sharePrice()` or a `deposit→withdraw`
  no-profit round-trip, then fuzz that).
- If it truly can't be reduced, keep it **`REASONED`** and, in `proof:`, give
  the concrete hand-traced values. Say plainly it wasn't machine-verified.
  Honesty here is the whole point — never label it `VERIFIED`.

## Deployed / unverified targets

If the target is a live contract (no source), the same engine verifies against
fetched bytecode:

```bash
truent fuzz --dynamic --address 0x<contract> --rpc-url <https-endpoint>
```

Truent fetches the runtime bytecode, probes it against known ERC20/Ownable
selectors, and fuzzes the confirmed surface. (It does not fork on-chain storage
— it exercises the contract's own accounting logic. Note this scope in the
finding.)

## The rule

The machine is the arbiter. LLM confidence never promotes a finding to
`VERIFIED`; only an engine reproduction does. An engine that can't break a
claimed invariant, after a real search, demotes it. This is exactly the step a
prompt-only tool structurally cannot perform — it has no engine to appeal to.
