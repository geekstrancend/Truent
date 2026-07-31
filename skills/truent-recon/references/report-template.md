# Recon report template (`recon/recon.md`)

Fill every section. Keep prose tight; tables over paragraphs. Cite `file:line`.

```markdown
# <Protocol> — Truent Recon

_Engine: static ✓ · dynamic {✓|skipped} · chain: <evm|solana|move|soroban>_
_Git signal from <N> commits, <M> contributors._

## 1. Overview
<2–4 sentences, plain language: what it does, who uses it, what value it holds.>

## 2. Threat model
- **Assets at risk:** <tokens, collateral, fees, governance power…>
- **Actors & trust:**
  | Actor | Permissions | Trust level |
  |-------|-------------|-------------|
  | user | … | untrusted |
  | owner/admin | … | trusted |
- **Attack surfaces (priority-ordered by git risk):**
  | Surface | Why risky | Git signal |
  |---------|-----------|------------|
- **External dependencies:** <oracles, tokens, bridges, libs> — and what the
  code assumes about each.
- **Temporal / upgrade risk:** <upgradeable? timelock? mutable params mid-flow?>

## 3. Entry points
| Function | Caller | Mutates | Value moves | Priority |
|----------|--------|---------|-------------|----------|
<externally-callable functions; priority = on a git hotspot ? high : normal>

## 4. Invariants
| Invariant | Kind | Engine result |
|-----------|------|---------------|
| sum(balanceOf) == totalSupply | conservation · engine-checkable | held / **BROKEN** |
| sharePrice() never decreases | monotonic · engine-checkable | … |
| only owner() sets owner() | access-control · engine-checkable | … |
| <doc-stated property> | doc-stated · manual | to verify |

For any **BROKEN** row, paste the fuzzer's minimal PoC.

## 5. Git-history risk map
_From `recon/git-security.json`._
- **Fix candidates** (historically bug-prone): <file — N fix commits>
- **Churn hotspots** (high defect density): <file — N changes, fix-ratio>
- **Late changes** (recently touched, least reviewed): <files>
- **Forked deps** (drift from upstream): <files or "none">
- **Tech-debt markers:** <count> — notable: <file:line — marker>

## 6. Test & coverage gaps
<what tests exist; which hot files lack coverage; property/invariant tests?>

## 7. Prioritized audit plan
1. <file:function> — <why first: engine finding / top hotspot / late change>
2. …
> Hand this list to `truent-audit` for the deep, engine-verified pass.
```
