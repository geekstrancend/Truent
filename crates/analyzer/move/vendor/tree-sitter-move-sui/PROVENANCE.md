# Vendored from MystenLabs/sui

These files are vendored (not a git dependency) from the official Sui
Move tree-sitter grammar, maintained by the Sui team as part of their
own Move tooling.

- Source: https://github.com/MystenLabs/sui
- Path: `external-crates/move/tooling/tree-sitter`
- Commit: `518ef64ab85405c0ad028714112183cad69a26f2`
- Fetched: 2026-07-14
- License: MIT (see `NOTICE`)

## Why vendored instead of a git dependency

`MystenLabs/sui` is a very large monorepo; even a shallow,
tree-filtered clone (`--filter=tree:0 --no-checkout`) took ~50s, and a
full sparse-checkout of just this directory took another ~2 minutes.
Depending on it live via `git = "https://github.com/MystenLabs/sui"`
would impose that cost on every fresh clone and CI cache miss, just to
get one grammar. Vendoring the specific files needed avoids that
entirely, at the cost of needing a manual re-sync (repeat this
process against a newer commit) to pick up upstream grammar fixes.

## What's included

- `grammar.js` — the grammar source (hand-maintained, drives
  everything else)
- `src/` — solver-generated parser output (`parser.c`,
  `node-types.json`, `grammar.json`) as committed upstream, used as a
  reference; Truent regenerates its own `bindings/rust/*` and
  `src/parser.c` locally via `tree-sitter generate` so the ABI matches
  whatever `tree-sitter` crate version this workspace pins, rather
  than trusting upstream's pre-generated output to match.
- `queries/` — syntax-highlighting queries (not currently used by
  Truent, kept for completeness/future use)
- `tests/` — upstream's own corpus of real-world `.move` snippets,
  useful as a smoke-test corpus beyond Truent's own fixtures

## Upstream status

Upstream's own README describes this as a **work-in-progress**
grammar with "no guarantees" on parsing all valid Move code. Treat it
as a best-effort structural parser, not a validating compiler
front-end - Truent's detectors should degrade gracefully (as they
already do for solc-dependent EVM analysis) if a real-world Move file
fails to parse.
