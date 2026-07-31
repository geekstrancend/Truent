# Attacker lenses (Phase 2 roster)

Truent's engine already ran the mechanical detectors in Phase 1, so the LLM
lenses do **not** re-scan for reentrancy-shape, overflow-shape, missing-signer,
etc. — that's wasted tokens on solved ground. Instead, five sharp lenses hunt
the *residual*: bugs that live in this protocol's bespoke logic. Fewer, deeper
lenses beat a broad shallow sweep.

Each lens: read the whole in-scope source once, apply `methodology.md`
continuously, and for every suspected bug **state the invariant it breaks**
(so Phase 3 can hand it to the engine). Output in the `judging.md` block format,
tier `REASONED`.

---

### Lens 1 — Economic / value extraction

You extract value the accounting didn't intend. Pricing curves, fee math,
reward accrual, share/asset conversion, liquidation payouts, AMM invariants.
Hunt: rounding that compounds or rounds to zero in the attacker's favor;
first/last-actor asymmetries; fee applied on the wrong base; slippage/deadline
gaps; a `deposit → withdraw` round-trip that returns more than it took; MEV in
mutable mid-flight parameters. **Property to state:** the conservation or
no-profit round-trip you believe is violated.

### Lens 2 — Invariant & state coupling

You break relationships that must hold. When X changes, Y must change too —
find every writer of X and the one that forgets Y. Capacity caps enforced on
`deposit()` but skipped on settlement/fee-accrual/emergency paths. `view`
functions that promise a number the state-changing twin computes differently
(penalty/accrual omitted from the view). Cache-then-mutate-then-use of the same
storage slot. **Property to state:** the conservation law, coupling, cap, or
view/write equivalence — these map most directly to engine checks, so this
lens has the highest verify rate.

### Lens 3 — Access & trust boundaries

You do what only a privileged actor should. Missing or tautological auth
(`require(msg.sender == msg.sender)`), unprotected initializers, functions that
change ownership/roles/critical params without the guard, and — the subtle one
— privileged actions that create an *unprivileged* amplifier (a race window, a
retroactive sweep of already-credited value). **Property to state:** the access
rule (e.g. only `owner()` may mutate `owner()`), which the engine can fuzz
directly.

### Lens 4 — Cross-function & sequencing

You exploit order and multi-call flows. Reentrancy beyond the classic CEI
(cross-contract, read-only, via callbacks/hooks). Commutativity breaks
(`A();B()` ≠ `B();A()`). Timestamp/cooldown resets via a secondary path. Global
parameters mutated during an in-flight multi-block operation (draws, vault
settlement). Emergency transitions that strand value because cleanup is
incomplete. **Property to state:** the call sequence that must not be
profitable, or the trace pattern (re-entry + late write) the engine's inspector
catches.

### Lens 5 — Integration & external assumptions

You break what the contract assumes about the outside world. Oracle staleness /
missing `updatedAt` / single-source price. Token quirks the code ignores:
fee-on-transfer, rebasing, blacklist, non-standard return, 6-decimals. Unchecked
external-call return values. Cross-chain / bridge message validation (source
chain + sender allowlist), and DVN/attestation single-point-of-failure.
**Property to state:** the assumption (e.g. "balance received == amount sent")
that a real token behavior violates.
