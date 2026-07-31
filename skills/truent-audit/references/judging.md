# Validation gates, dedup, and report format

## Finding tiers (assign to every finding)

- **`VERIFIED`** — the Truent engine produced or reproduced it (static detector
  match, or a dynamic fuzz violation with a PoC). Reproducible by anyone.
- **`REASONED`** — an LLM lens found it and the engine could not (or could not
  yet) confirm it by execution. Report it, clearly labeled, with what remained
  unverified.

`VERIFIED` findings **skip Gate 1–3** — the engine already demonstrated
execution. They still get Gate 4 (impact framing) for the writeup. `REASONED`
findings run all four gates.

## The four gates (REASONED findings)

Evaluate in order. Fail any gate → **rejected** or **demoted to lead**. You are
verifying the attacker's claim fires end-to-end, not defending the code.

1. **Attack execution.** Trace the claimed path from caller to harm. Read every
   guard/modifier/check on it. If a *specific* one interrupts the exploit before
   harm (quote the line) → **reject** (or demote if a related smell remains). A
   *speculative* interruption ("the caller would notice", "the deployer would
   set X") does **not** save the code → clears.
2. **Reachability.** Prove the vulnerable state can exist in a live deployment.
   Structurally impossible (an enforced invariant prevents it) → reject. Needs
   privileged setup → demote. Reachable through normal use or common token
   behavior (fee-on-transfer, rebasing, blacklisting are all plausible) → clears.
3. **Trigger.** Prove an *unprivileged* actor sets it off. Only trusted roles
   can → demote. **Admin-does-admin-things is not a finding** — reject unless the
   body names a concrete unprivileged amplifier (a race window, a retroactive
   sweep of already-credited value, an asymmetric formula an unprivileged actor
   profits from, or a missing/tautological access guard).
4. **Impact.** Prove material harm to an identifiable victim. Self-harm only →
   reject. Dust with no compounding → demote. Material loss → **confirmed**.

## Do not report

Linter/compiler noise, gas micro-opts, naming, missing events, NatSpec.
Centralization "admin can rug" without a concrete unprivileged mechanism.
Safe-by-design patterns: `unchecked` in 0.8+ (verify the reasoning), SafeERC20,
`nonReentrant` (only cross-contract reentrancy counts), two-step ownership,
MINIMUM_LIQUIDITY first-deposit burn, consistent protocol-favoring rounding
(unless it compounds or rounds to zero).

## Dedup

Group by `(contract, function, bug-class)`. Never merge across different
functions — a different function is a different bug. Within a function, if two
findings need *different fixes*, keep them separate and present each fix. If a
`VERIFIED` and a `REASONED` finding share a root cause, merge and keep
`VERIFIED`.

## Report format

Header (one line):

```
Truent Audit — N verified · M reasoned · engine: static ✓, dynamic {✓|skipped}
```

Then findings, highest severity first (VERIFIED above REASONED at equal
severity), each:

```
[VERIFIED|REASONED]  <SEVERITY>  <title>
  where:   <contract>.<function>  ·  <file:line>
  class:   <kebab-bug-class>   (id: <invariant_id if from a detector>)
  what:    one-sentence root cause
  impact:  who loses what, concretely
  proof:   concrete values / trace  — for VERIFIED dynamic findings, the
           minimal PoC call sequence Truent emitted
  fix:     the smallest change that removes the defect (a diff when possible)
```

Footer — the reproduce line, so the reader can re-run the proven parts and get
the identical result:

```
Reproduce:  truent scan <path> --chain <chain>
            truent fuzz <file> --dynamic --chain evm --seed <seed>   # for dynamic findings
```
