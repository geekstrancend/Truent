#!/usr/bin/env bash
# engine.sh — run Truent's deterministic engine and emit machine-verified
# findings as JSON. This is the ground-truth layer of a truent-audit run:
# every finding here is produced by a compiled analyzer, not an LLM, so it
# is reproducible and free of hallucination.
#
# Usage:
#   engine.sh <path> [chain]
#     <path>   file or directory to scan
#     [chain]  evm | solana | move | soroban   (default: evm)
#
# Output (stdout): the JSON object from `truent scan --output json`, with a
# leading `_truent` block describing engine provenance. Exit code mirrors
# the scan's --fail-on (non-zero when critical findings exist), so this
# doubles as a CI gate.
set -uo pipefail

PATH_ARG="${1:?usage: engine.sh <path> [chain]}"
CHAIN="${2:-evm}"

# ── Locate the truent binary ─────────────────────────────────────────────
# Prefer an installed `truent`; fall back to a build in this repo.
find_truent() {
  if command -v truent >/dev/null 2>&1; then command -v truent; return; fi
  local here; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  # Walk up looking for a cargo target dir (skill may live inside the repo,
  # or be installed alongside it).
  local d="$here"
  for _ in 1 2 3 4 5 6; do
    for cand in "$d/target/release/truent" "$d/target/debug/truent"; do
      [ -x "$cand" ] && { echo "$cand"; return; }
    done
    d="$(dirname "$d")"
  done
  # Also try the current working directory's target.
  for cand in "./target/release/truent" "./target/debug/truent"; do
    [ -x "$cand" ] && { echo "$cand"; return; }
  done
  return 1
}

TRUENT="$(find_truent)" || {
  echo '{"_truent":{"error":"truent binary not found. Install it (cargo install --path crates/cli) or build it (cargo build --release --bin truent)."}}' >&2
  exit 127
}

# ── Run the deterministic scan ───────────────────────────────────────────
# --output json is stable, machine-parseable, and identical run-to-run.
# --fail-on critical makes the exit code a usable gate signal.
"$TRUENT" scan "$PATH_ARG" --chain "$CHAIN" --output json --fail-on critical
