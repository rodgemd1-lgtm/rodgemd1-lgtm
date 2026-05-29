# Makefile — rodgemd1-lgtm
# Deterministic local targets. No secrets, deployments, or network required for smoke/doctor.

REPO_SLUG := rodgemd1-lgtm
BIN       := bin/$(REPO_SLUG)
SHELL     := /bin/bash

.PHONY: smoke doctor proof help

## smoke: Run the deterministic local smoke check (exit 0 = pass)
smoke:
	@bash scripts/smoke.sh

## doctor: Check local dependencies and remote reachability
doctor:
	@bash $(BIN) doctor

## proof: Write/refresh the proof document
proof:
	@bash scripts/write-proof.sh

## help: Show available targets
help:
	@grep -E '^## ' Makefile | sed 's/^## /  /'
