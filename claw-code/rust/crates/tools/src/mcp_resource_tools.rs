// PR 1: MCP Resource Access Tools
// File: rust/crates/tools/src/mcp_resource_tools.rs
//
// Adds ListMcpResources and ReadMcpResource tools to the claw-code tool registry.
// These wire into the existing McpServerManager in runtime/src/mcp_stdio.rs.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// --- Tool Specs (add to mvp_tool_specs() in lib.rs) ---

pub fn mcp_resource_tool_specs() -> Vec<super::ToolSpec> {
    vec![
        super::ToolSpec {
            name: "ListMcpResources",
            description: "List available resources from configured MCP servers. Each resource includes a URI, name, description, and MIME type.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": {
                        "type": "string",
                        "description": "Optional server name to filter resources by. If omitted, resources from all servers are returned."
                    }
                },
                "additionalProperties": false
            }),
            required_permission: super::PermissionMode::ReadOnly,
        },
        super::ToolSpec {
            name: "ReadMcpResource",
            description: "Read a specific resource from an MCP server, identified by server name and resource URI.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": {
                        "type": "string",
                        "description": "The name of the MCP server to read from."
                    },
                    "uri": {
                        "type": "string",
                        "description": "The URI of the resource to read."
                    }
                },
                "required": ["server", "uri"],
                "additionalProperties": false
            }),
            required_permission: super::PermissionMode::ReadOnly,
        },
    ]
}

// --- Input Types ---

#[derive(Debug, Deserialize)]
pub struct ListMcpResourcesInput {
    pub server: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadMcpResourceInput {
    pub server: String,
    pub uri: String,
}

// --- Output Types ---

#[derive(Debug, Serialize)]
pub struct McpResourceEntry {
    pub server: String,
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct McpResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub text: String,
}

// --- Execution Handlers ---
//
// These handlers require access to the MCP server manager at runtime.
// Integration approach: The GlobalToolRegistry.execute() method should be
// extended to accept an optional McpServerManager reference, or a
// process-global MCP registry should be used (matching the existing pattern
// for tools that need runtime state).
//
// For now, these are standalone functions that accept the manager as a parameter.

/// Trait abstracting MCP resource operations for testability.
pub trait McpResourceProvider: Send + Sync {
    fn list_resources(
        &self,
        server: Option<&str>,
    ) -> Result<Vec<McpResourceEntry>, String>;

    fn read_resource(
        &self,
        server: &str,
        uri: &str,
    ) -> Result<McpResourceContent, String>;
}

pub fn run_list_mcp_resources(
    input: ListMcpResourcesInput,
    provider: &dyn McpResourceProvider,
) -> Result<String, String> {
    let resources = provider.list_resources(input.server.as_deref())?;
    serde_json::to_string_pretty(&resources).map_err(|e| e.to_string())
}

pub fn run_read_mcp_resource(
    input: ReadMcpResourceInput,
    provider: &dyn McpResourceProvider,
) -> Result<String, String> {
    let content = provider.read_resource(&input.server, &input.uri)?;
    serde_json::to_string_pretty(&content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMcpProvider;

    impl McpResourceProvider for MockMcpProvider {
        fn list_resources(
            &self,
            server: Option<&str>,
        ) -> Result<Vec<McpResourceEntry>, String> {
            let all = vec![
                McpResourceEntry {
                    server: "test-server".to_string(),
                    uri: "file:///readme.md".to_string(),
                    name: "README".to_string(),
                    description: Some("Project readme".to_string()),
                    mime_type: Some("text/markdown".to_string()),
                },
                McpResourceEntry {
                    server: "other-server".to_string(),
                    uri: "db://users".to_string(),
                    name: "Users table".to_string(),
                    description: None,
                    mime_type: None,
                },
            ];
            match server {
                Some(name) => Ok(all.into_iter().filter(|r| r.server == name).collect()),
                None => Ok(all),
            }
        }

        fn read_resource(
            &self,
            server: &str,
            uri: &str,
        ) -> Result<McpResourceContent, String> {
            if server == "test-server" && uri == "file:///readme.md" {
                Ok(McpResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("text/markdown".to_string()),
                    text: "# Hello World".to_string(),
                })
            } else {
                Err(format!("resource not found: {server}/{uri}"))
            }
        }
    }

    #[test]
    fn list_all_resources() {
        let provider = MockMcpProvider;
        let result = run_list_mcp_resources(
            ListMcpResourcesInput { server: None },
            &provider,
        )
        .unwrap();
        assert!(result.contains("test-server"));
        assert!(result.contains("other-server"));
    }

    #[test]
    fn list_filtered_resources() {
        let provider = MockMcpProvider;
        let result = run_list_mcp_resources(
            ListMcpResourcesInput {
                server: Some("test-server".to_string()),
            },
            &provider,
        )
        .unwrap();
        assert!(result.contains("test-server"));
        assert!(!result.contains("other-server"));
    }

    #[test]
    fn read_existing_resource() {
        let provider = MockMcpProvider;
        let result = run_read_mcp_resource(
            ReadMcpResourceInput {
                server: "test-server".to_string(),
                uri: "file:///readme.md".to_string(),
            },
            &provider,
        )
        .unwrap();
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn read_missing_resource_errors() {
        let provider = MockMcpProvider;
        let result = run_read_mcp_resource(
            ReadMcpResourceInput {
                server: "unknown".to_string(),
                uri: "missing".to_string(),
            },
            &provider,
        );
        assert!(result.is_err());
    }
}
