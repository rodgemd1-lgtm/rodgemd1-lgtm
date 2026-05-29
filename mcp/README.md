# MCP Posture — rodgemd1-lgtm

## Status: Intentionally Deferred (V3 — plan only)

This repo (`rodgemd1-lgtm`) is a **public GitHub profile and identity surface**.
At V3 of the roadmap, no MCP server or client is implemented here.

MCP integration will be considered at **V4+** once the following prerequisites are met:

1. A concrete tool surface is identified (e.g., a command that yields structured data
   useful to an agent client).
2. Auth boundaries and no-secret constraints are confirmed for any tool handler.
3. The weekly automation loop is stable and passing smoke checks.

---

## Planned MCP Surface (V4 candidate)

When MCP is activated, the following surface is planned:

### Tools

| Tool | Description | Auth required |
|------|-------------|---------------|
| `repo.info` | Return repo identity, lane, and source-of-truth routing | None |
| `repo.capabilities` | Return the capabilities list from `cli/manifest.json` | None |
| `repo.doctor` | Run local dependency checks and return structured results | None |

### Resources

| Resource | URI pattern | Description |
|----------|-------------|-------------|
| `cli-manifest` | `rig://rodgemd1-lgtm/cli/manifest.json` | Read the CLI manifest |
| `proof-log` | `rig://rodgemd1-lgtm/proof/looper-v10-cockpit/weekly-run.log` | Latest proof run log |

### Prompts

| Prompt | Description |
|--------|-------------|
| `v10-design-review` | Ask the agent to review the current V10 design document |
| `smoke-check` | Ask the agent to run and interpret the smoke check output |

---

## Boundaries

- No MCP tool may expose QNAP SSH credentials, Tailscale hostnames, or tokens.
- No MCP tool may write files, deploy, send messages, or activate schedules.
- All tools are read-only until a sealed DoneContract authorizes write operations.

---

## Manifest File (future)

When MCP is activated, a `mcp/manifest.json` will be created here with the schema:

```json
{
  "schema": "rig.mcp.v1",
  "name": "rodgemd1-lgtm",
  "tools": [],
  "resources": [],
  "prompts": [],
  "auth": "none"
}
```
