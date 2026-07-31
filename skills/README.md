# Truent Skills

Deterministic-first security skills for AI coding agents (Claude Code, Cursor,
Codex, Copilot, Windsurf). Unlike prompt-only audit skills, these are backed by
[Truent](https://github.com/geekstrancend/Truent)'s **compiled engine** — static
analyzers plus a real `revm`-backed invariant fuzzer — so findings are
machine-verified and reproducible, not an LLM's opinion.

| Skill | What it does |
|-------|--------------|
| [truent-audit](truent-audit/) | Deterministic-first, engine-verified smart-contract audit across EVM · Solana · Move · Soroban. Runs the compiled engine first, then amplifies with LLM attacker lenses whose findings are verified back through the engine before being reported. |
| [truent-recon](truent-recon/) | Pre-audit recon: git-history risk mining + threat model + entry points + synthesized invariants — with the checkable invariants run through the engine, not just listed. |
| [truent-fuzz](truent-fuzz/) | Stateful invariant fuzzing. Native revm-backed fuzzer (auto-detected invariants + minimal-PoC shrinking, no external toolchain) by default; emits an equivalent Echidna/Medusa harness on demand. |

## Prerequisite

The skills call the `truent` binary. Install it once:

```bash
# from a clone of the Truent repo
cargo install --path crates/cli
# or build in-place
cargo build --release --bin truent
```

Verify: `truent doctor` should report all components healthy.

## Install (Claude Code)

```
Install https://github.com/geekstrancend/Truent and run truent-audit on the codebase
```

Or copy `skills/truent-audit/` into your agent's skills directory.
