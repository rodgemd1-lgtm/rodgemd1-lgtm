// PR 5: Agent Orchestration
// File: rust/crates/tools/src/agent_registry.rs
//
// Agent type registry with allowed tool sets and system prompt suffixes.
// Supports parallel agent execution via thread spawning.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

// ============================================================================
// Agent Type Registry
// ============================================================================

/// Specification for a named agent type with constrained tool access.
#[derive(Debug, Clone)]
pub struct AgentTypeSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub allowed_tools: &'static [&'static str],
    pub system_prompt_suffix: &'static str,
}

/// Returns all built-in agent types matching Claude Code's agent system.
pub fn builtin_agent_types() -> Vec<AgentTypeSpec> {
    vec![
        AgentTypeSpec {
            name: "general-purpose",
            description: "General-purpose agent for research, code search, and multi-step tasks.",
            allowed_tools: &[
                "bash", "read_file", "write_file", "edit_file", "glob_search", "grep_search",
                "WebFetch", "WebSearch", "TodoWrite", "Skill", "Agent", "NotebookEdit",
                "ToolSearch", "ListMcpResources", "ReadMcpResource",
            ],
            system_prompt_suffix: "You are a general-purpose agent. Complete the task autonomously and return the result.",
        },
        AgentTypeSpec {
            name: "Explore",
            description: "Fast agent for codebase exploration. Read-only — never modifies files.",
            allowed_tools: &[
                "bash", "read_file", "glob_search", "grep_search",
                "WebFetch", "WebSearch", "ListMcpResources", "ReadMcpResource",
            ],
            system_prompt_suffix: "You are an exploration agent. Search the codebase and report findings. Do NOT modify any files.",
        },
        AgentTypeSpec {
            name: "Plan",
            description: "Architecture consultant for designing implementation plans. Read-only.",
            allowed_tools: &[
                "bash", "read_file", "glob_search", "grep_search",
                "WebFetch", "WebSearch",
            ],
            system_prompt_suffix: "You are a planning agent. Design implementation strategies and identify critical files. Do NOT modify any files.",
        },
        AgentTypeSpec {
            name: "Verification",
            description: "Agent for running tests, builds, and verifying changes.",
            allowed_tools: &[
                "bash", "read_file", "glob_search", "grep_search",
            ],
            system_prompt_suffix: "You are a verification agent. Run tests, builds, and linters. Report results clearly. Do NOT modify source files.",
        },
        AgentTypeSpec {
            name: "CodeWriter",
            description: "Agent focused on writing and editing code files.",
            allowed_tools: &[
                "bash", "read_file", "write_file", "edit_file", "glob_search", "grep_search",
            ],
            system_prompt_suffix: "You are a code writing agent. Implement changes precisely as described.",
        },
    ]
}

/// Look up an agent type by name (case-insensitive).
pub fn find_agent_type(name: &str) -> Option<AgentTypeSpec> {
    let lower = name.to_ascii_lowercase();
    builtin_agent_types()
        .into_iter()
        .find(|spec| spec.name.to_ascii_lowercase() == lower)
}

/// Get the allowed tool set for a given agent type.
/// Returns None for unknown types (caller should use default full access).
pub fn allowed_tools_for_agent_type(agent_type: &str) -> Option<BTreeSet<String>> {
    find_agent_type(agent_type).map(|spec| {
        spec.allowed_tools
            .iter()
            .map(|s| s.to_string())
            .collect()
    })
}

// ============================================================================
// Parallel Agent Execution
// ============================================================================

/// Input for a single agent task in a parallel batch.
#[derive(Debug, Clone, Deserialize)]
pub struct ParallelAgentTask {
    pub description: String,
    pub prompt: String,
    #[serde(default = "default_agent_type")]
    pub subagent_type: String,
}

fn default_agent_type() -> String {
    "general-purpose".to_string()
}

/// Result from a single agent task.
#[derive(Debug, Clone, Serialize)]
pub struct ParallelAgentResult {
    pub description: String,
    pub subagent_type: String,
    pub output: String,
    pub is_error: bool,
}

/// Execute multiple agent tasks in parallel using threads.
///
/// `agent_runner` is a function that takes (prompt, agent_type) and returns
/// the agent's output string (or error).
pub fn run_parallel_agents<F>(
    tasks: Vec<ParallelAgentTask>,
    agent_runner: F,
) -> Vec<ParallelAgentResult>
where
    F: Fn(&str, &str) -> Result<String, String> + Send + Sync + 'static,
{
    use std::sync::Arc;
    let runner = Arc::new(agent_runner);

    let handles: Vec<_> = tasks
        .into_iter()
        .map(|task| {
            let runner = Arc::clone(&runner);
            let description = task.description.clone();
            let subagent_type = task.subagent_type.clone();
            let prompt = task.prompt.clone();

            std::thread::spawn(move || {
                match runner(&prompt, &subagent_type) {
                    Ok(output) => ParallelAgentResult {
                        description,
                        subagent_type,
                        output,
                        is_error: false,
                    },
                    Err(error) => ParallelAgentResult {
                        description,
                        subagent_type,
                        output: error,
                        is_error: true,
                    },
                }
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|handle| {
            handle.join().unwrap_or_else(|_| ParallelAgentResult {
                description: "unknown".to_string(),
                subagent_type: "unknown".to_string(),
                output: "agent thread panicked".to_string(),
                is_error: true,
            })
        })
        .collect()
}

// ============================================================================
// Enhanced Agent Tool Spec
// ============================================================================

/// Extended Agent tool spec that supports both single and parallel execution.
pub fn agent_tool_spec() -> super::ToolSpec {
    super::ToolSpec {
        name: "Agent",
        description: "Launch one or more specialized agents to handle tasks autonomously. Supports parallel execution for independent tasks.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Short description of what the agent will do (3-5 words)."
                },
                "prompt": {
                    "type": "string",
                    "description": "The task for the agent to perform."
                },
                "subagent_type": {
                    "type": "string",
                    "description": "Agent type: general-purpose, Explore, Plan, Verification, CodeWriter.",
                    "default": "general-purpose"
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "Run in background (non-blocking). Default false.",
                    "default": false
                },
                "parallel_tasks": {
                    "type": "array",
                    "description": "Multiple tasks to run in parallel. When provided, description/prompt/subagent_type are ignored.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": { "type": "string" },
                            "prompt": { "type": "string" },
                            "subagent_type": { "type": "string", "default": "general-purpose" }
                        },
                        "required": ["description", "prompt"]
                    }
                }
            },
            "additionalProperties": false
        }),
        required_permission: super::PermissionMode::ReadOnly,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_known_agent_types() {
        assert!(find_agent_type("Explore").is_some());
        assert!(find_agent_type("explore").is_some()); // case-insensitive
        assert!(find_agent_type("Plan").is_some());
        assert!(find_agent_type("general-purpose").is_some());
        assert!(find_agent_type("nonexistent").is_none());
    }

    #[test]
    fn explore_agent_is_read_only() {
        let spec = find_agent_type("Explore").unwrap();
        assert!(!spec.allowed_tools.contains(&"write_file"));
        assert!(!spec.allowed_tools.contains(&"edit_file"));
        assert!(spec.allowed_tools.contains(&"read_file"));
        assert!(spec.allowed_tools.contains(&"grep_search"));
    }

    #[test]
    fn parallel_agents_execute() {
        let tasks = vec![
            ParallelAgentTask {
                description: "task1".to_string(),
                prompt: "do thing 1".to_string(),
                subagent_type: "Explore".to_string(),
            },
            ParallelAgentTask {
                description: "task2".to_string(),
                prompt: "do thing 2".to_string(),
                subagent_type: "Plan".to_string(),
            },
        ];

        let results = run_parallel_agents(tasks, |prompt, agent_type| {
            Ok(format!("executed {agent_type}: {prompt}"))
        });

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_error);
        assert!(!results[1].is_error);
        assert!(results[0].output.contains("Explore"));
        assert!(results[1].output.contains("Plan"));
    }

    #[test]
    fn parallel_agent_error_handling() {
        let tasks = vec![ParallelAgentTask {
            description: "failing task".to_string(),
            prompt: "fail".to_string(),
            subagent_type: "Explore".to_string(),
        }];

        let results = run_parallel_agents(tasks, |_prompt, _agent_type| {
            Err("intentional failure".to_string())
        });

        assert_eq!(results.len(), 1);
        assert!(results[0].is_error);
        assert!(results[0].output.contains("intentional failure"));
    }

    #[test]
    fn allowed_tools_returns_correct_set() {
        let tools = allowed_tools_for_agent_type("Explore").unwrap();
        assert!(tools.contains("read_file"));
        assert!(tools.contains("grep_search"));
        assert!(!tools.contains("write_file"));

        let tools = allowed_tools_for_agent_type("CodeWriter").unwrap();
        assert!(tools.contains("write_file"));
        assert!(tools.contains("edit_file"));
    }
}
