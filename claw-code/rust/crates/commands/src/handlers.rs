// PR 6: Command Handler Implementations
// File: rust/crates/commands/src/handlers.rs
//
// Flesh out /bughunter, /ultraplan, /teleport command handlers.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================================
// /bughunter — Scan workspace for common bug patterns
// ============================================================================

#[derive(Debug, Clone)]
pub struct BugReport {
    pub file: String,
    pub line: usize,
    pub category: &'static str,
    pub severity: &'static str,
    pub message: String,
}

/// Scan the workspace for common bug patterns using grep.
pub fn handle_bughunter_command(scope: Option<&str>, cwd: &Path) -> io::Result<String> {
    let target = scope.unwrap_or(".");
    let target_path = cwd.join(target);

    if !target_path.exists() {
        return Ok(format!("Path not found: {}", target_path.display()));
    }

    let mut bugs = Vec::new();

    // Pattern categories to scan
    let patterns: Vec<(&str, &str, &str)> = vec![
        // (pattern, category, severity)
        ("TODO|FIXME|HACK|XXX|WORKAROUND", "tech-debt", "info"),
        ("unwrap\\(\\)", "error-handling", "warning"),
        ("panic!\\(", "error-handling", "warning"),
        ("unsafe\\s*\\{", "safety", "warning"),
        ("expect\\(\"", "error-handling", "info"),
        ("#\\[allow\\(unused", "dead-code", "info"),
        ("println!\\(", "debug-leftover", "info"),
        ("dbg!\\(", "debug-leftover", "warning"),
        ("eprintln!\\(\"DEBUG", "debug-leftover", "warning"),
        ("\\.clone\\(\\)\\s*\\.clone\\(\\)", "performance", "warning"),
    ];

    for (pattern, category, severity) in &patterns {
        if let Ok(output) = Command::new("grep")
            .args(["-rn", "--include=*.rs", "--include=*.py", "--include=*.ts",
                   "--include=*.js", "--include=*.go", pattern])
            .arg(&target_path)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().take(50) {
                // Parse grep output: file:line:content
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                if parts.len() >= 3 {
                    let file = parts[0]
                        .strip_prefix(&cwd.to_string_lossy().to_string())
                        .unwrap_or(parts[0])
                        .trim_start_matches('/');
                    let line_num: usize = parts[1].parse().unwrap_or(0);
                    let content = parts[2].trim();

                    bugs.push(BugReport {
                        file: file.to_string(),
                        line: line_num,
                        category,
                        severity,
                        message: content.chars().take(120).collect(),
                    });
                }
            }
        }
    }

    if bugs.is_empty() {
        return Ok("No bug patterns found in the scanned scope.".to_string());
    }

    // Group by severity
    let errors: Vec<_> = bugs.iter().filter(|b| b.severity == "error").collect();
    let warnings: Vec<_> = bugs.iter().filter(|b| b.severity == "warning").collect();
    let infos: Vec<_> = bugs.iter().filter(|b| b.severity == "info").collect();

    let mut output = String::new();
    output.push_str(&format!("# Bug Hunter Report\n\n"));
    output.push_str(&format!(
        "Scanned: `{}`\nFound: {} patterns ({} warnings, {} info)\n\n",
        target,
        bugs.len(),
        warnings.len(),
        infos.len()
    ));

    if !warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for bug in warnings.iter().take(25) {
            output.push_str(&format!(
                "- **{}:{}** [{}] {}\n",
                bug.file, bug.line, bug.category, bug.message
            ));
        }
        output.push('\n');
    }

    if !infos.is_empty() {
        output.push_str("## Info\n\n");
        for bug in infos.iter().take(25) {
            output.push_str(&format!(
                "- {}:{} [{}] {}\n",
                bug.file, bug.line, bug.category, bug.message
            ));
        }
    }

    Ok(output)
}

// ============================================================================
// /ultraplan — Generate a structured implementation plan
// ============================================================================

/// Generate a structured planning prompt for the given task.
/// This constructs a system message that, when sent to the conversation runtime
/// with the Agent tool (type=Plan), produces a step-by-step implementation plan.
pub fn handle_ultraplan_command(task: Option<&str>, cwd: &Path) -> io::Result<String> {
    let task_description = task.unwrap_or("No task specified. Describe what you want to build.");

    let plan_prompt = format!(
        r#"You are an expert software architect. Create a detailed implementation plan for the following task.

## Task
{task_description}

## Working Directory
{cwd}

## Plan Format
Produce a structured plan with:

1. **Context** — What problem does this solve? What prompted it?
2. **Architecture** — Key design decisions, patterns, and trade-offs
3. **Files to Create/Modify** — Specific file paths with descriptions
4. **Implementation Steps** — Ordered checklist with dependencies noted
5. **Testing Strategy** — How to verify each step
6. **Risks** — What could go wrong and mitigations

Keep each step small, reviewable, and independently testable.
Use checkboxes (- [ ]) for actionable items.
"#,
        cwd = cwd.display()
    );

    Ok(plan_prompt)
}

// ============================================================================
// /teleport — Navigate to a symbol or file
// ============================================================================

/// Search for a symbol, function, or file path in the workspace.
/// Returns matching locations sorted by relevance.
pub fn handle_teleport_command(target: Option<&str>, cwd: &Path) -> io::Result<String> {
    let target = match target {
        Some(t) if !t.is_empty() => t,
        _ => return Ok("Usage: /teleport <symbol-or-path>\n\nExamples:\n  /teleport MyStruct\n  /teleport src/main.rs\n  /teleport fn run_server".to_string()),
    };

    let mut results = Vec::new();

    // 1. Try exact file path match
    let file_path = cwd.join(target);
    if file_path.exists() {
        results.push(format!("**Exact match:** `{target}`"));
    }

    // 2. Glob for file name matches
    if let Ok(output) = Command::new("find")
        .args([
            cwd.to_str().unwrap_or("."),
            "-name",
            &format!("*{}*", target.split('/').last().unwrap_or(target)),
            "-not", "-path", "*/target/*",
            "-not", "-path", "*/.git/*",
            "-not", "-path", "*/node_modules/*",
            "-type", "f",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(10) {
            let relative = line
                .strip_prefix(&cwd.to_string_lossy().to_string())
                .unwrap_or(line)
                .trim_start_matches('/');
            if !relative.is_empty() {
                results.push(format!("**File:** `{relative}`"));
            }
        }
    }

    // 3. Grep for symbol definitions (fn, struct, class, def, type, interface)
    let def_patterns = [
        format!("(fn|struct|enum|trait|type|impl|mod)\\s+{target}"),
        format!("(class|def|function|interface|const|let|var)\\s+{target}"),
    ];

    for pattern in &def_patterns {
        if let Ok(output) = Command::new("grep")
            .args([
                "-rn", "--include=*.rs", "--include=*.py", "--include=*.ts",
                "--include=*.js", "--include=*.go", "--include=*.java",
                "-E", pattern,
            ])
            .arg(cwd)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().take(10) {
                let parts: Vec<&str> = line.splitn(3, ':').collect();
                if parts.len() >= 3 {
                    let file = parts[0]
                        .strip_prefix(&cwd.to_string_lossy().to_string())
                        .unwrap_or(parts[0])
                        .trim_start_matches('/');
                    let line_num = parts[1];
                    let content = parts[2].trim();
                    results.push(format!(
                        "**Definition:** `{file}:{line_num}` — `{}`",
                        content.chars().take(80).collect::<String>()
                    ));
                }
            }
        }
    }

    // 4. Grep for usage/references
    if let Ok(output) = Command::new("grep")
        .args([
            "-rn", "--include=*.rs", "--include=*.py", "--include=*.ts",
            "--include=*.js", "-w", target,
        ])
        .arg(cwd)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let ref_count = stdout.lines().count();
        if ref_count > 0 {
            results.push(format!("**References:** {ref_count} occurrences across the workspace"));
        }
    }

    if results.is_empty() {
        Ok(format!("No matches found for `{target}` in {}", cwd.display()))
    } else {
        let mut output = format!("# Teleport: `{target}`\n\n");
        for result in results {
            output.push_str(&format!("{result}\n"));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn bughunter_on_empty_dir() {
        let dir = std::env::temp_dir().join("bughunter_test_empty");
        let _ = fs::create_dir_all(&dir);
        let result = handle_bughunter_command(None, &dir).unwrap();
        assert!(result.contains("No bug patterns") || result.contains("Bug Hunter"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bughunter_finds_todo() {
        let dir = std::env::temp_dir().join("bughunter_test_todo");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("test.rs"), "fn main() {\n    // TODO: fix this\n}\n").unwrap();

        let result = handle_bughunter_command(None, &dir).unwrap();
        assert!(result.contains("TODO") || result.contains("tech-debt"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ultraplan_generates_prompt() {
        let cwd = Path::new("/tmp/test_project");
        let result = handle_ultraplan_command(Some("Add user authentication"), cwd).unwrap();
        assert!(result.contains("Add user authentication"));
        assert!(result.contains("Implementation Steps"));
        assert!(result.contains("Testing Strategy"));
    }

    #[test]
    fn teleport_no_target_shows_usage() {
        let cwd = Path::new("/tmp");
        let result = handle_teleport_command(None, cwd).unwrap();
        assert!(result.contains("Usage:"));
    }

    #[test]
    fn teleport_finds_exact_file() {
        let dir = std::env::temp_dir().join("teleport_test");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("main.rs"), "fn main() {}").unwrap();

        let result = handle_teleport_command(Some("main.rs"), &dir).unwrap();
        assert!(result.contains("main.rs"));

        let _ = fs::remove_dir_all(&dir);
    }
}
