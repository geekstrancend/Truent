# Truent

[![crates.io](https://img.shields.io/crates/v/truent-cli.svg)](https://crates.io/crates/truent-cli)
[![npm](https://img.shields.io/npm/v/@dextonicx/cli.svg)](https://www.npmjs.com/package/@dextonicx/cli)
[![Downloads](https://img.shields.io/crates/d/truent-cli.svg)](https://crates.io/crates/truent-cli)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/geekstrancend/Truent/actions/workflows/ci.yml/badge.svg)](https://github.com/geekstrancend/Truent/actions)

**Multi-chain smart contract security analyzer for EVM, Solana, Move, and Soroban.**

Truent checks your smart contracts and programs for vulnerabilities before
deployment. Define what should always be true — invariants — and Truent
verifies your code cannot violate them.

One tool. Four chains. One DSL.

---

## What's new in v0.3.0

v0.3.0 reconnects the full detection pipeline into the CLI and adds a
chain-agnostic detection layer on top of it.

**Key improvements:**

- ✅ **71 Smart Contract Vulnerability Detectors** — Comprehensive coverage of critical and high-priority exploits, wired end-to-end into `truent check`/`truent scan`
- ✅ **Chain-agnostic shared rule** — `unauthorized_privileged_mutation` runs against a common semantic model built by each chain's own analyzer, so one rule (missing an authorization check on a privileged mutation) is written once and applies to all four chains
- ✅ **Real Move parsing** — a vendored Sui Move tree-sitter grammar backs Move's semantic extraction, with the original regex heuristic kept as an automatic fallback if a file fails to parse
- ✅ **Soroban (Stellar) support** — a fourth full chain analyzer covering `require_auth` gaps, unprotected contract upgrades, re-initialization, unchecked arithmetic, storage TTL/expiry, and reentrancy-shaped checks-effects-interactions violations
- ✅ **Real fuzzing** — `truent fuzz` mutates real source files and runs them through the live detectors looking for crashes, instead of a no-op stub
- ✅ **Production Ready** — All tests passing, security audit complete, reproducible builds

**Detector Coverage:**
- **EVM**: 44 detectors (reentrancy incl. read-only reentrancy, missing checks, oracle manipulation incl. stale-price feeds, proxy issues, insufficient multisig threshold, arbitrary function-selector dispatch, fee-on-transfer/rebasing incompatibility, cross-chain signature replay, unbounded pricing input, ERC-4337 validation side effects, EIP-7702 EOA assumptions, and 20+ named historical-exploit patterns)
- **Solana**: 11 detectors (PDA validation, authority checks, replay attacks, durable nonce, rent exemption, unchecked token/mint account substitution, fake sysvar instructions account)
- **Move**: 7 detectors (resource destruction, type safety, access control, hand-rolled overflow checks)
- **Soroban**: 9 detectors (missing require_auth, unprotected upgrade, re-initialization, unchecked arithmetic, storage TTL/expiry, reentrancy, thin-liquidity oracle price)

Truent also ships a web dashboard (`web/`) — sign-up, scan submission, and
report viewing on top of the same CLI engine — alongside the `truent` CLI
and its npm wrapper (`@dextonicx/cli`).

---

## What's new in v0.2.2

v0.2.2 adds reproducibility and flexible output options for better CI integration and reporting.

**Key improvements:**

- ✅ **Reproducible analysis** — `--seed` flag for deterministic results across runs
- ✅ **File output** — `--output` flag to write reports to disk (text, JSON, HTML)
- ✅ **HTML reports** — Beautiful formatted security reports with styled tables and summaries
- ✅ **Solana SDK 1.x** — Updated to latest stable Solana SDK

---

## What's new in v0.2.1

v0.2.1 fixes violation location reporting — all violations now show their actual source line numbers instead of defaulting to line 1. This dramatically improves debugging workflow.

**Key improvements:**

- ✅ **Accurate violation locations** — Real line numbers from the vulnerable code
- ✅ **Code context** — Shows 2 lines before/after violation for quick reference
- ✅ **Embedded line tracking** — Line numbers calculated during AST analysis, not post-processing

---

## What's new in v0.2.0

v0.2 replaces pattern matching with real Rust AST parsing via the `syn`
crate. Truent now understands Anchor's type system and eliminates false
positives on idiomatic Anchor programs.

| Pattern | v0.1 | v0.2 |
| --- | --- | --- |
| `Signer<'info>` | ❌ False positive | ✅ Correctly silent |
| `Account<'info, T>` | ❌ Over-flagged | ✅ Recognized as safe |
| `AccountInfo` with `seeds = [...]` | ❌ False positive | ✅ Correctly silent |
| `AccountInfo` with `/// CHECK:` | ❌ False positive | ✅ Downgraded to INFO |
| `AccountInfo` with no constraint | ✅ CRITICAL | ✅ Still CRITICAL |

> **Upgrading from v0.1/v0.2.0?** Run `cargo install truent-cli --force`

---

## Install

```bash
# Rust developers
cargo install truent-cli

# JavaScript / TypeScript developers
npm install -g @dextonicx/cli

# Verify installation
truent --version   # truent 0.4.1
truent doctor
```

Or download a pre-built binary directly from
[GitHub Releases](https://github.com/geekstrancend/Truent/releases).

**Supported platforms:**

- Linux x86_64, aarch64, musl
- macOS x86_64, aarch64 (Apple Silicon)
- Windows x86_64

---

## Use it on your codebase

Three steps: install, point Truent at your repo, gate your CI.

**1. Install** (see above) — `cargo install truent-cli` or `npm install -g @dextonicx/cli`.

**2. Scan your code.** Run from your project root and point Truent at the files
you want checked. `--chain` defaults to `evm`; set it for other ecosystems:

```bash
truent scan .                          # Solidity / EVM (the default)
truent scan ./programs  --chain solana
truent scan ./sources   --chain move
truent scan ./contracts --chain soroban
```

Each finding is printed with its severity, file and line, and the real-world
exploit pattern it maps to. Want a report to share or feed into other tools?

```bash
truent scan . --format html --output truent-report.html   # styled, shareable
truent scan . --format json --output truent-report.json   # machine-readable
```

**3. Gate your CI.** Make a risky change fail the build:

```bash
truent scan . --chain evm --fail-on high   # exit non-zero on High/Critical
```

Drop that into GitHub Actions and you're done:

```yaml
- name: Truent security scan
  run: |
    cargo install truent-cli
    truent scan . --chain evm --fail-on high
```

`truent scan --help` lists every option; `truent doctor` verifies your install.

---

## Quick start

```bash
# Check a Solana program
truent scan ./programs --chain solana

# Check Solidity contracts
truent scan ./contracts --chain evm

# Check Move modules
truent scan ./sources --chain move

# Check Soroban contracts
truent scan ./contracts --chain soroban

# Output as JSON
truent scan ./programs --chain solana --format json

# Output as HTML
truent scan ./programs --chain solana --format html --output ./report.html

# Reproducible analysis with fixed seed
truent scan ./programs --chain solana --seed 42

# Fail CI if high or critical violations found
truent scan ./programs --chain solana --fail-on high

# Run health check
truent doctor

# Initialize config
truent init
```

---

## Output options

### Report formats

```bash
# Text report (default, human-readable)
truent scan ./programs --chain solana --format text

# JSON report (machine-readable, for parsing/CI)
truent scan ./programs --chain solana --format json

# HTML report (styled, shareable with team)
truent scan ./programs --chain solana --format html
```

### Saving reports to disk

```bash
# Save any format to file
truent scan ./programs --chain solana --format json --output ./report.json
truent scan ./programs --chain solana --format html --output ./report.html
truent scan ./programs --chain solana --format text --output ./report.txt
```

### Reproducible analysis

For deterministic results (useful in CI or security audits), use `--seed`:

```bash
# Always uses seed 42 by default
truent scan ./programs --chain solana

# Use a custom seed
truent scan ./programs --chain solana --seed 12345

# Results will be identical on the same code with the same seed
```

---

## GitHub Actions

Add one step to your workflow:

```yaml
- name: Truent security check
  run: |
    cargo install truent-cli
    truent scan ./programs --chain solana --fail-on high
```

CI fails automatically on high or critical violations. Zero additional
configuration required.

---

## Built-in invariants

Truent ships with 28 built-in security checks across all four chains.

### EVM (10 invariants)

| ID | Name | Severity |
| --- | --- | --- |
| `evm_reentrancy_protection` | Reentrancy Protection | Critical |
| `evm_integer_overflow` | Integer Overflow | High |
| `evm_integer_underflow` | Integer Underflow | High |
| `evm_unchecked_returns` | Unchecked Return Values | Medium |
| `evm_delegatecall_injection` | Delegatecall Injection | Critical |
| `evm_access_control` | Access Control | High |
| `evm_timestamp_dependence` | Timestamp Dependence | Medium |
| `evm_frontrunning` | Front-running | Medium |
| `evm_uninitialized_pointers` | Uninitialized Pointers | High |
| `evm_division_by_zero` | Division by Zero | Medium |

### Solana (7 invariants)

| ID | Name | Severity |
| --- | --- | --- |
| `sol_signer_checks` | Signer Checks | Critical |
| `sol_account_validation` | Account Validation | Critical |
| `sol_integer_overflow` | Integer Overflow | High |
| `sol_rent_exemption` | Rent Exemption | Medium |
| `sol_pda_derivation` | PDA Derivation | High |
| `sol_lamport_balance` | Lamport Balance | Critical |
| `sol_instruction_parsing` | Instruction Parsing | Medium |

### Move (5 invariants)

| ID | Name | Severity |
| --- | --- | --- |
| `move_access_control` | Access Control | Critical |
| `move_integer_overflow` | Integer Overflow | High |
| `move_resource_leaks` | Resource Leaks | High |
| `move_type_safety` | Type Safety | High |
| `move_signer_requirement` | Signer Requirement | Critical |

### Soroban (6 invariants)

| ID | Name | Severity |
| --- | --- | --- |
| `sor_require_auth_checks` | Require-Auth Checks | High |
| `sor_no_unprotected_upgrade` | Protected Upgrade | High |
| `sor_init_guard` | Initializer Guard | High |
| `sor_checked_arithmetic` | Checked Arithmetic | High |
| `sor_storage_ttl_extended` | Storage TTL Extended | High |
| `sor_no_reentrancy` | No Reentrancy | High |

---

## Configuration

Create `.truent.toml` in your project root:

```toml
[project]
name = "my-project"
chain = "solana"

[analysis]
severity_threshold = "low"
# suppress = ["sol_rent_exemption"]

[output]
format = "text"   # text | json | html
```

Or run `truent init` to generate a config automatically.

### Inline suppression

```rust
// truent: ignore sol_account_validation — external VRF oracle account
pub oracle_queue: AccountInfo<'info>,
```

---

## Anchor false positive guide (v0.2+)

Truent v0.2 understands Anchor's type system. These patterns are
correctly handled:

```rust
// SAFE — Anchor enforces signer automatically
pub authority: Signer<'info>,

// SAFE — Anchor validates ownership and discriminator
pub arena: Account<'info, Arena>,

// SAFE — seeds constraint validates PDA derivation
#[account(seeds = [b"vault", user.key().as_ref()], bump)]
pub vault: AccountInfo<'info>,

// SAFE — developer has verified this external account
/// CHECK: This is the Switchboard VRF oracle. Address validated off-chain.
pub oracle_queue: AccountInfo<'info>,

// CRITICAL — genuinely unchecked, Truent correctly fires
pub mystery: AccountInfo<'info>,
```

---

## Roadmap

| Version | Focus | Status |
| --- | --- | --- |
| v0.1 | Pattern-based analysis, 22 invariants, full CLI | ✅ Shipped |
| v0.2 | Real AST parsing, Anchor-aware analysis | ✅ Shipped |
| v0.3 | Runtime fuzzing — revm + solana-program-test | 🔨 Next |
| v0.4 | Bounded model checking | 📋 Planned |
| v0.5 | Symbolic execution via Z3 | 📋 Planned |
| v1.0 | Slither + Echidna + Mythril for every chain | 🎯 Goal |

---

## Links

- **GitHub**: [geekstrancend/Truent](https://github.com/geekstrancend/Truent)
- **crates.io**: [truent-cli](https://crates.io/crates/truent-cli)
- **npm**: [@dextonicx/cli](https://www.npmjs.com/package/@dextonicx/cli)
- **Docs**: [docs.rs/truent-cli](https://docs.rs/truent-cli)

---

## Contributing

Issues, PRs, and feedback are welcome.

If you are a Rust engineer familiar with `syn` or Anchor internals,
the v0.3 fuzzing work is the highest-impact contribution area right now.

If you are a smart contract auditor, help expand the invariant library
with real attack patterns you have encountered.

---

## License

MIT — see [LICENSE](LICENSE)
