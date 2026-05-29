#!/usr/bin/env bash
# scripts/smoke.sh — deterministic local smoke check for rodgemd1-lgtm
# Safe to run without credentials, network access, or deployment authority.
# Exit 0 = all checks pass. Exit 1 = one or more checks failed.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PASS=0
FAIL=0

check() {
  local label="$1"
  local result="$2"
  if [ "$result" = "ok" ]; then
    printf '[PASS] %s\n' "$label"
    PASS=$((PASS + 1))
  else
    printf '[FAIL] %s — %s\n' "$label" "$result"
    FAIL=$((FAIL + 1))
  fi
}

# --- Required files ---
for f in README.md bin/rodgemd1-lgtm install.sh cli/manifest.json; do
  if [ -f "$REPO_ROOT/$f" ]; then
    check "file exists: $f" "ok"
  else
    check "file exists: $f" "missing"
  fi
done

# --- bin entrypoint is executable ---
if [ -x "$REPO_ROOT/bin/rodgemd1-lgtm" ]; then
  check "bin/rodgemd1-lgtm is executable" "ok"
else
  check "bin/rodgemd1-lgtm is executable" "not executable"
fi

# --- CLI manifest is valid JSON ---
if command -v python3 >/dev/null 2>&1; then
  if python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$REPO_ROOT/cli/manifest.json" 2>/dev/null; then
    check "cli/manifest.json is valid JSON" "ok"
  else
    check "cli/manifest.json is valid JSON" "invalid JSON"
  fi
else
  check "cli/manifest.json is valid JSON" "python3 not available — skipped"
fi

# --- Agent instructions exist ---
if [ -f "$REPO_ROOT/.github/copilot-instructions.md" ]; then
  check "agent instructions present" "ok"
else
  check "agent instructions present" "missing"
fi

# --- MCP posture declared ---
if [ -f "$REPO_ROOT/mcp/README.md" ]; then
  check "MCP posture declared" "ok"
else
  check "MCP posture declared" "missing"
fi

# --- Proof directory exists ---
if [ -d "$REPO_ROOT/proof/looper-v10-cockpit" ]; then
  check "proof directory exists" "ok"
else
  check "proof directory exists" "missing"
fi

# --- No secrets patterns (naive guard) ---
if grep -rI --include='*.sh' --include='*.json' --include='*.md' \
     -E '(ghp_[A-Za-z0-9]{36,}|ghs_[A-Za-z0-9]{36,}|sk-[A-Za-z0-9]{40,}|password\s*=\s*\S)' \
     "$REPO_ROOT" >/dev/null 2>&1; then
  check "no obvious secret patterns" "FOUND — review before commit"
else
  check "no obvious secret patterns" "ok"
fi

# --- Summary ---
printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
