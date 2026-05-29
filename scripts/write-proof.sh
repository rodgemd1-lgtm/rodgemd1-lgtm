#!/usr/bin/env bash
# scripts/write-proof.sh — write a sanitized proof document for the looper-v10-cockpit
# No secrets, private paths, or credentials are written to the proof file.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROOF_DIR="$REPO_ROOT/proof/looper-v10-cockpit"
PROOF_FILE="$PROOF_DIR/weekly-run.log"

mkdir -p "$PROOF_DIR"

TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
COMMIT_SHA="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo 'unknown')"

# Run smoke and capture result
if bash "$REPO_ROOT/scripts/smoke.sh" >"$PROOF_DIR/smoke-output.txt" 2>&1; then
  SMOKE_STATUS="PASS"
else
  SMOKE_STATUS="FAIL"
fi

cat >>"$PROOF_FILE" <<LOG
---
timestamp: $TIMESTAMP
commit: $COMMIT_SHA
smoke: $SMOKE_STATUS
---
LOG

printf 'Proof written: %s\nSmoke: %s\n' "$PROOF_FILE" "$SMOKE_STATUS"
cat "$PROOF_DIR/smoke-output.txt"
