use serde_json::{Value, json};

use super::{COMMAND_TOOL_NAME, QUESTION_TOOL_NAME, TEXT_QUESTION_TOOL_NAME};

pub(crate) fn openai_tools() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": COMMAND_TOOL_NAME,
                "description": "Return a single shell command for the user request.",
                "parameters": command_tool_schema()
            }
        },
        {
            "type": "function",
            "function": {
                "name": QUESTION_TOOL_NAME,
                "description": "Ask a required yes/no clarification question before generating a command.",
                "parameters": question_tool_schema()
            }
        },
        {
            "type": "function",
            "function": {
                "name": TEXT_QUESTION_TOOL_NAME,
                "description": "Ask a required free-text clarification question before generating a command.",
                "parameters": text_question_tool_schema()
            }
        }
    ])
}

pub(crate) fn gemini_function_declarations() -> Value {
    json!([
        {
            "name": COMMAND_TOOL_NAME,
            "description": "Return a single shell command for the user request.",
            "parameters": command_tool_schema()
        },
        {
            "name": QUESTION_TOOL_NAME,
            "description": "Ask a required yes/no clarification question before generating a command.",
            "parameters": question_tool_schema()
        },
        {
            "name": TEXT_QUESTION_TOOL_NAME,
            "description": "Ask a required free-text clarification question before generating a command.",
            "parameters": text_question_tool_schema()
        }
    ])
}

pub(crate) fn claude_tools() -> Value {
    json!([
        {
            "name": COMMAND_TOOL_NAME,
            "description": "Return a single shell command for the user request.",
            "input_schema": command_tool_schema()
        },
        {
            "name": QUESTION_TOOL_NAME,
            "description": "Ask a required yes/no clarification question before generating a command.",
            "input_schema": question_tool_schema()
        },
        {
            "name": TEXT_QUESTION_TOOL_NAME,
            "description": "Ask a required free-text clarification question before generating a command.",
            "input_schema": text_question_tool_schema()
        }
    ])
}

fn command_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "Single shell command line."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for the chosen command."
            },
            "explanations": {
                "type": "array",
                "description": "Optional explanation items when explanation mode is enabled.",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": {"type": "string"},
                        "value": {"type": "string"},
                        "explanation": {"type": "string"}
                    },
                    "required": ["type", "value", "explanation"]
                }
            }
        },
        "required": ["command", "reason"]
    })
}

fn question_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "question": {
                "type": "string",
                "description": "One clear yes/no question."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for asking this clarification."
            }
        },
        "required": ["question", "reason"]
    })
}

fn text_question_tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "question": {
                "type": "string",
                "description": "One clear question to collect a concrete value from user."
            },
            "reason": {
                "type": "string",
                "description": "Short reason for asking this clarification."
            }
        },
        "required": ["question", "reason"]
    })
}
