# V10 Design — rodgemd1-lgtm

**Generated:** 2026-05-29  
**Commit:** (see `proof/looper-v10-cockpit/weekly-run.log` for per-run SHA)  
**Status:** Planning and safe setup only. No sealed DoneContract. No production implementation.

---

## V10 Product Promise

`rodgemd1-lgtm` is the **public GitHub identity surface** for Mike Rodgers / Rodgers Intelligence Group (RIG).

At V10 it will be:
- A clean, self-documenting, deterministic repo that passes all local smoke checks without credentials.
- An installable CLI (`rodgemd1-lgtm`) that exposes repo identity, capabilities, and source-of-truth routing.
- An agent-ready repo with visible instructions, model-routing expectations, and quality gates.
- A proof-producing repo with weekly automation health records in `proof/looper-v10-cockpit/`.

---

## CLI Surface

**Command:** `rodgemd1-lgtm`  
**Binary:** `bin/rodgemd1-lgtm`  
**Manifest:** `cli/manifest.json`  
**Installer:** `install.sh`

### Subcommands

| Command | Description |
|---------|-------------|
| `info` | Repo identity and source-of-truth routing |
| `capabilities` | Capability contract |
| `services` | Declared run/test/build commands |
| `clone` | Clone or update the source repo under `~/.rig/repos` |
| `open` | Open local clone or GitHub |
| `docs` | Open the GitHub README |
| `doctor` | Check dependencies and remote reachability |
| `help` | Usage |

### First Deterministic Smoke Command

```bash
make smoke
```

Or directly:

```bash
bash scripts/smoke.sh
```

This checks required files, executable permissions, JSON validity, agent instructions,
MCP posture, and proof directory presence. No network or credentials required.

---

## MCP Surface

**Status:** Intentionally deferred at V3. See `mcp/README.md`.

Planned for V4+:
- `repo.info` tool (read-only, no auth)
- `repo.capabilities` tool (read-only, no auth)
- `repo.doctor` tool (read-only, no auth)
- `cli-manifest` resource
- `proof-log` resource

---

## Agent Roles and Model Routing

| Role | Scope | Model |
|------|-------|-------|
| Planner | Issue → design plan → proof doc | Sonnet |
| Reviewer | PR diff review, boundary enforcement | Sonnet |
| Fixer | Targeted file patches | Haiku |
| QA | `make smoke`, `make doctor`, proof validation | Haiku |

**Routing rules:**
- Sonnet for design and review.
- Haiku for deterministic execution.
- No private QNAP paths or credentials through any external model.

---

## Quality Gates

1. `make smoke` exits 0 (all file and format checks pass).
2. `make doctor` reports `git: ok` and `python3: ok`.
3. `proof/looper-v10-cockpit/` contains current design doc and run log.
4. No secrets, tokens, or private paths in committed files.
5. No direct pushes to `main`; PRs require human approval for `README.md` and `bin/`.

---

## Weekly Improvement Loop

1. CI runs `make smoke` on every push.
2. Weekly automation appends a run record to `proof/looper-v10-cockpit/weekly-run.log`.
3. Blockers are filed as GitHub issues; nothing is silently dropped.
4. Looper may propose updates via PR; **human must approve merges**.
5. Never runs deploys, publishes, or schedule activations automatically.

---

## Blockers

| Blocker | Type | Safe next action |
|---------|------|-----------------|
| No CI workflow yet | Missing setup | Add `.github/workflows/smoke.yml` (manual human action) |
| QNAP reachability from CI | Network | Use GitHub fallback clone in CI |
| MCP not implemented | Deferred | Activate at V4 with a DoneContract |
| No weekly automation schedule | Deferred | Wire Looper schedule (human approval required) |

---

## Missing Items

- `.github/workflows/smoke.yml` — CI smoke check workflow (requires human to create)
- `cli/manifest.json` service_commands populated (smoke/doctor added in this PR)
- Weekly automation schedule (human approval required to activate)
- MCP manifest (deferred to V4)

---

## Proof Paths

| Artifact | Path |
|----------|------|
| This design doc | `proof/looper-v10-cockpit/v10-design.md` |
| Weekly run log | `proof/looper-v10-cockpit/weekly-run.log` |
| Smoke output | `proof/looper-v10-cockpit/smoke-output.txt` |
| CLI manifest | `cli/manifest.json` |
| Smoke script | `scripts/smoke.sh` |
| Agent instructions | `.github/copilot-instructions.md` |
| MCP deferral | `mcp/README.md` |

---

## Scores (projected after this PR)

| KPI | Before | After |
|-----|--------|-------|
| setup_git | 10/10 | 10/10 |
| agent_readiness | 2/10 | 7/10 |
| cli_readiness | 3/10 | 7/10 |
| mcp_readiness | 0/10 | 3/10 |
| quality_readiness | 0/10 | 5/10 |
| proof_readiness | 2/10 | 7/10 |
| weekly_automation_readiness | 6/10 | 7/10 |
| **v10_current_score** | **3/10** | **~6/10** |

Note: Final PASS is not claimed. CI and weekly automation wiring requires human approval.
