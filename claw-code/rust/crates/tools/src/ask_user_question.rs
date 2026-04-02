// PR 2: AskUserQuestion Tool
// File: rust/crates/tools/src/ask_user_question.rs
//
// Interactive question tool that lets the model present structured questions
// to the user and receive their response. Integrates with the PermissionPrompter
// trait in runtime/src/conversation.rs.

use serde::{Deserialize, Serialize};
use serde_json::json;

// --- Tool Spec (add to mvp_tool_specs() in lib.rs) ---

pub fn ask_user_question_tool_spec() -> super::ToolSpec {
    super::ToolSpec {
        name: "AskUserQuestion",
        description: "Ask the user a question during execution. Use to gather preferences, clarify ambiguous instructions, get decisions on implementation choices, or offer choices.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "description": "Questions to ask the user (1-4 questions).",
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The complete question to ask."
                            },
                            "header": {
                                "type": "string",
                                "description": "Short label displayed as a chip/tag (max 12 chars)."
                            },
                            "options": {
                                "type": "array",
                                "description": "Available choices (2-4 options). An 'Other' option is auto-added.",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": { "type": "string" },
                                        "description": { "type": "string" }
                                    },
                                    "required": ["label", "description"]
                                },
                                "minItems": 2,
                                "maxItems": 4
                            },
                            "multiSelect": {
                                "type": "boolean",
                                "default": false,
                                "description": "Allow multiple selections."
                            }
                        },
                        "required": ["question", "header", "options", "multiSelect"]
                    },
                    "minItems": 1,
                    "maxItems": 4
                }
            },
            "required": ["questions"],
            "additionalProperties": false
        }),
        required_permission: super::PermissionMode::ReadOnly,
    }
}

// --- Input Types ---

#[derive(Debug, Clone, Deserialize)]
pub struct AskUserQuestionInput {
    pub questions: Vec<QuestionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionSpec {
    pub question: String,
    pub header: String,
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub multi_select: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
}

// --- Output Types ---

#[derive(Debug, Clone, Serialize)]
pub struct AskUserQuestionOutput {
    pub answers: std::collections::BTreeMap<String, String>,
}

// --- User Responder Trait ---
//
// This trait should be implemented by the CLI's interactive prompter.
// The PermissionPrompter trait in conversation.rs is the natural integration
// point — extend it with an ask_user() method, or create this as a separate
// trait that the CLI implements alongside PermissionPrompter.

pub trait UserQuestionResponder: Send {
    /// Present a question to the user and return their answer.
    /// For single-select: returns the selected option label or custom text.
    /// For multi-select: returns comma-separated selected labels.
    fn ask(&mut self, question: &QuestionSpec) -> Result<String, String>;
}

// --- Execution Handler ---

pub fn run_ask_user_question(
    input: AskUserQuestionInput,
    responder: &mut dyn UserQuestionResponder,
) -> Result<String, String> {
    let mut answers = std::collections::BTreeMap::new();

    for question_spec in &input.questions {
        let answer = responder.ask(question_spec)?;
        answers.insert(question_spec.question.clone(), answer);
    }

    let output = AskUserQuestionOutput { answers };
    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

// --- CLI Implementation ---
//
// This is a reference implementation for terminal-based interaction.
// Drop this into claw-cli/src/main.rs or a dedicated module.

pub struct TerminalQuestionResponder;

impl UserQuestionResponder for TerminalQuestionResponder {
    fn ask(&mut self, question: &QuestionSpec) -> Result<String, String> {
        use std::io::{self, BufRead, Write};

        eprintln!("\n[{}] {}", question.header, question.question);
        for (i, option) in question.options.iter().enumerate() {
            eprintln!("  {}. {} - {}", i + 1, option.label, option.description);
        }
        let other_idx = question.options.len() + 1;
        eprintln!("  {other_idx}. Other (custom input)");

        if question.multi_select {
            eprint!("Select one or more (comma-separated numbers): ");
        } else {
            eprint!("Select: ");
        }
        io::stderr().flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;

        let selections: Vec<usize> = line
            .trim()
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .collect();

        if selections.is_empty() {
            return Ok(line.trim().to_string());
        }

        let mut labels = Vec::new();
        for sel in selections {
            if sel >= 1 && sel <= question.options.len() {
                labels.push(question.options[sel - 1].label.clone());
            } else if sel == other_idx {
                eprint!("Enter custom response: ");
                io::stderr().flush().map_err(|e| e.to_string())?;
                let mut custom = String::new();
                io::stdin()
                    .lock()
                    .read_line(&mut custom)
                    .map_err(|e| e.to_string())?;
                labels.push(custom.trim().to_string());
            }
        }

        Ok(labels.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockResponder {
        responses: Vec<String>,
        call_count: usize,
    }

    impl UserQuestionResponder for MockResponder {
        fn ask(&mut self, _question: &QuestionSpec) -> Result<String, String> {
            if self.call_count < self.responses.len() {
                let response = self.responses[self.call_count].clone();
                self.call_count += 1;
                Ok(response)
            } else {
                Err("no more scripted responses".to_string())
            }
        }
    }

    #[test]
    fn single_question_returns_answer() {
        let input = AskUserQuestionInput {
            questions: vec![QuestionSpec {
                question: "Which language?".to_string(),
                header: "Language".to_string(),
                options: vec![
                    QuestionOption {
                        label: "Rust".to_string(),
                        description: "Systems language".to_string(),
                    },
                    QuestionOption {
                        label: "Python".to_string(),
                        description: "Scripting language".to_string(),
                    },
                ],
                multi_select: false,
            }],
        };

        let mut responder = MockResponder {
            responses: vec!["Rust".to_string()],
            call_count: 0,
        };

        let result = run_ask_user_question(input, &mut responder).unwrap();
        assert!(result.contains("Rust"));
        assert!(result.contains("Which language?"));
    }

    #[test]
    fn multiple_questions_returns_all_answers() {
        let input = AskUserQuestionInput {
            questions: vec![
                QuestionSpec {
                    question: "Framework?".to_string(),
                    header: "Framework".to_string(),
                    options: vec![
                        QuestionOption {
                            label: "Axum".to_string(),
                            description: "Async web".to_string(),
                        },
                        QuestionOption {
                            label: "Actix".to_string(),
                            description: "Actor web".to_string(),
                        },
                    ],
                    multi_select: false,
                },
                QuestionSpec {
                    question: "Database?".to_string(),
                    header: "DB".to_string(),
                    options: vec![
                        QuestionOption {
                            label: "Postgres".to_string(),
                            description: "Relational".to_string(),
                        },
                        QuestionOption {
                            label: "SQLite".to_string(),
                            description: "Embedded".to_string(),
                        },
                    ],
                    multi_select: false,
                },
            ],
        };

        let mut responder = MockResponder {
            responses: vec!["Axum".to_string(), "Postgres".to_string()],
            call_count: 0,
        };

        let result = run_ask_user_question(input, &mut responder).unwrap();
        assert!(result.contains("Axum"));
        assert!(result.contains("Postgres"));
    }
}
