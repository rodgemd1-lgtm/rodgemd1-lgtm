# Copilot Agent Instructions — rodgemd1-lgtm

## Repo Identity

- **Name:** rodgemd1-lgtm
- **Lane:** public-web — public GitHub profile and identity surface for Mike Rodgers / Rodgers Intelligence Group
- **Visibility:** public
- **Canonical source:** QNAP Gitea (`ssh://git@nas94f2ae.tail4d96b3.ts.net:2222/rig/rodgemd1-lgtm.git`)
- **GitHub mirror:** `https://github.com/rodgemd1-lgtm/rodgemd1-lgtm`

## V10 Product Promise

This repo is the public-facing GitHub identity surface for Mike Rodgers / Rodgers Intelligence Group (RIG). Its V10 goal is to be a clean, deterministic, self-documenting profile repo that:

1. Exposes a CLI for local inspection, cloning, and routing.
2. Declares its MCP posture (tools, resources, prompts, or intentional deferral).
3. Provides proof of weekly-automation health via `proof/looper-v10-cockpit/`.
4. Passes all local smoke checks without secrets, deployments, or human approval.

## Agent Roles

| Role | Scope | Model routing |
|------|-------|---------------|
| Planner | Reads issue, generates design plan, writes proof doc | Sonnet |
| Reviewer | Reviews PR diffs, flags security/boundary violations | Sonnet |
| Fixer | Applies targeted patches, never rewrites history | Haiku |
| QA | Runs `make smoke` and `make doctor`, validates proof output | Haiku |

## Model-Routing Expectations

- Use Sonnet for design, planning, and review decisions.
- Use Haiku for deterministic script execution and file-level patches.
- Never route private QNAP paths, tokens, or credentials through any external model call.
- Prefer local deterministic checks before any agentic action.

## Quality Gates

1. `make smoke` must exit 0 before any PR is merged.
2. `make doctor` must report `git: ok` and `python3: ok`.
3. `proof/looper-v10-cockpit/` must contain an up-to-date design doc.
4. No secrets, tokens, cookies, or private paths may appear in committed files.

## Boundaries — What Agents Must Never Do

- Do not push directly to `main`; always use a branch and PR.
- Do not deploy, publish, send messages, or activate schedules.
- Do not reset, clean, delete, or overwrite user work.
- Do not expose QNAP SSH keys, Tailscale hostnames, or private credentials.
- Do not merge PRs without at least one human approval when the change touches `README.md` or `bin/`.

## Weekly Improvement Loop

- The weekly automation runs `make smoke` and `make doctor` in CI.
- Results are appended to `proof/looper-v10-cockpit/weekly-run.log` (sanitized, no secrets).
- Blockers are filed as GitHub issues; they are never silently dropped.
- Looper may propose README or capability updates via PR; human must approve merges.

## Safe First Command

```bash
make smoke
```

This is the single deterministic entry point. It is cheap, local, and produces a
pass/fail signal without network access or credentials.

## MCP Posture

See `mcp/README.md`. MCP tooling is intentionally deferred at V3. The MCP manifest
will be populated once a tool surface is defined (V4+).

## Proof Paths

| Artifact | Path |
|----------|------|
| V10 design | `proof/looper-v10-cockpit/v10-design.md` |
| Weekly run log | `proof/looper-v10-cockpit/weekly-run.log` |
| CLI manifest | `cli/manifest.json` |
| Smoke script | `scripts/smoke.sh` |
