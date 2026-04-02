# Claw Code Gap Fix — Integration Guide

This directory contains 6 new Rust source files that close the remaining gaps
in the claw-code Rust port. Each file is self-contained with tests and can be
applied as an independent PR.

## File Inventory

| File | PR | Description |
|------|----|----|
| `rust/crates/tools/src/mcp_resource_tools.rs` | PR 1 | ListMcpResources + ReadMcpResource tools |
| `rust/crates/tools/src/ask_user_question.rs` | PR 2 | AskUserQuestion interactive tool |
| `rust/crates/runtime/src/scheduler.rs` | PR 3 | CronCreate/Delete/List session scheduler |
| `rust/crates/server/src/structured_io.rs` | PR 4 | JSON stdin/stdout protocol for IDE extensions |
| `rust/crates/tools/src/agent_registry.rs` | PR 5 | Agent type registry + parallel execution |
| `rust/crates/commands/src/handlers.rs` | PR 6 | /bughunter, /ultraplan, /teleport handlers |

---

## PR 1: MCP Resource Tools

### Add to `rust/crates/tools/src/lib.rs`:

1. Add module declaration at top:
```rust
pub mod mcp_resource_tools;
```

2. In `mvp_tool_specs()`, append:
```rust
// ... existing specs ...
]
.into_iter()
.chain(mcp_resource_tools::mcp_resource_tool_specs())
.collect()
```

3. In `execute_tool()`, add match arms:
```rust
"ListMcpResources" => {
    // Requires McpResourceProvider — wire through GlobalToolRegistry.execute()
    Err("ListMcpResources requires MCP server manager (not available in static context)".to_string())
}
"ReadMcpResource" => {
    Err("ReadMcpResource requires MCP server manager (not available in static context)".to_string())
}
```

4. In `GlobalToolRegistry::execute()`, add before the plugin fallback:
```rust
if name == "ListMcpResources" || name == "ReadMcpResource" {
    // Route to MCP-aware executor (pass McpServerManager reference)
    // This requires adding an optional McpResourceProvider field to GlobalToolRegistry
    // or threading it through a separate execution path.
    return Err(format!("{name} requires MCP context"));
}
```

**Full integration:** Add `mcp_provider: Option<Arc<dyn McpResourceProvider>>` field to
`GlobalToolRegistry` and route these tool calls through it.

---

## PR 2: AskUserQuestion Tool

### Add to `rust/crates/tools/src/lib.rs`:

1. Module declaration:
```rust
pub mod ask_user_question;
```

2. In `mvp_tool_specs()`, add the spec:
```rust
ask_user_question::ask_user_question_tool_spec(),
```

3. In `execute_tool()`, add:
```rust
"AskUserQuestion" => {
    Err("AskUserQuestion requires interactive prompter (not available in static context)".to_string())
}
```

### Add to `rust/crates/runtime/src/conversation.rs`:

Extend `PermissionPrompter` or create a new trait method:
```rust
pub trait UserInteraction: PermissionPrompter {
    fn ask_question(&mut self, question: &ask_user_question::QuestionSpec) -> Result<String, String> {
        // Default: return error (non-interactive mode)
        Err("interactive questions not supported in this mode".to_string())
    }
}
```

In the tool execution loop, detect AskUserQuestion and route to the prompter:
```rust
"AskUserQuestion" if prompter.is_some() => {
    let input: AskUserQuestionInput = serde_json::from_str(&input)?;
    let prompter = prompter.as_mut().unwrap();
    // Route to interactive handler
}
```

### Add to `rust/crates/claw-cli/src/main.rs`:

Implement `UserQuestionResponder` for the terminal prompter (reference
implementation provided in the file).

---

## PR 3: Scheduling Tools

### Add to `rust/crates/runtime/src/lib.rs`:

```rust
pub mod scheduler;
```

### Add to `rust/crates/tools/src/lib.rs`:

1. In `mvp_tool_specs()`, append the cron specs:
```rust
// Chain cron tool specs
.chain(crate::runtime::scheduler::cron_tool_specs())
```

Or add them directly since they're defined in the runtime crate — you may need
to re-export them or define the specs in the tools crate.

2. In `execute_tool()`:
```rust
"CronCreate" | "CronDelete" | "CronList" => {
    Err(format!("{name} requires session scheduler (not available in static context)"))
}
```

### Wire in `rust/crates/claw-cli/src/main.rs`:

Create a `SessionScheduler` alongside the `ConversationRuntime` and pass it
to a custom `ToolExecutor` wrapper that routes cron tools to the scheduler.

---

## PR 4: Structured IO Transport

### Add to `rust/crates/server/src/lib.rs`:

```rust
pub mod structured_io;
```

### Add to `rust/crates/claw-cli/src/main.rs`:

Add CLI flag:
```rust
#[arg(long, value_name = "FORMAT")]
output_format: Option<String>, // "json" for structured IO
```

When `output_format == Some("json")`:
```rust
use server::structured_io::{StdioJsonTransport, StructuredInput, StructuredOutput};
let mut transport = StdioJsonTransport::new();
// Run structured IO loop instead of TUI REPL
```

---

## PR 5: Agent Orchestration

### Add to `rust/crates/tools/src/lib.rs`:

```rust
pub mod agent_registry;
```

### Replace the Agent tool spec in `mvp_tool_specs()`:

Replace the existing Agent `ToolSpec` with:
```rust
agent_registry::agent_tool_spec(),
```

### In `execute_tool()` Agent handler:

Add parallel task support:
```rust
"Agent" => {
    let input: Value = serde_json::from_str(input)?;
    if let Some(parallel_tasks) = input.get("parallel_tasks") {
        let tasks: Vec<ParallelAgentTask> = serde_json::from_value(parallel_tasks.clone())?;
        let results = run_parallel_agents(tasks, |prompt, agent_type| {
            run_agent_inner(prompt, agent_type)
        });
        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    } else {
        // Existing single-agent path
        from_value::<AgentInput>(&input).and_then(run_agent)
    }
}
```

### Use agent type registry in SubagentToolExecutor:

Replace hardcoded tool filtering with:
```rust
use agent_registry::allowed_tools_for_agent_type;

fn build_subagent_tools(agent_type: &str) -> BTreeSet<String> {
    allowed_tools_for_agent_type(agent_type)
        .unwrap_or_else(|| /* default full tool set */)
}
```

---

## PR 6: Command Handlers

### Add to `rust/crates/commands/src/lib.rs`:

```rust
pub mod handlers;
```

### Wire into existing command dispatch:

In the slash command handler (likely in `claw-cli/src/main.rs` or `commands/src/lib.rs`):

```rust
SlashCommand::Bughunter => {
    let scope = args.first().map(|s| s.as_str());
    handlers::handle_bughunter_command(scope, &cwd)?
}
SlashCommand::Ultraplan => {
    let task = args.first().map(|s| s.as_str());
    handlers::handle_ultraplan_command(task, &cwd)?
}
SlashCommand::Teleport => {
    let target = args.first().map(|s| s.as_str());
    handlers::handle_teleport_command(target, &cwd)?
}
```

---

## Verification

For each PR, from `rust/`:

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Individual module tests:

```bash
cargo test --lib mcp_resource_tools
cargo test --lib ask_user_question
cargo test --lib scheduler
cargo test --lib structured_io
cargo test --lib agent_registry
cargo test --lib handlers
```
