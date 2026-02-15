use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationReport {
    pub syntax_ok: bool,
    pub shell: String,
    #[serde(default)]
    pub missing_binaries: Vec<String>,
    #[serde(default)]
    pub checked_binaries: Vec<String>,
    #[serde(default)]
    pub alias_conflicts: Vec<String>,
    #[serde(default)]
    pub invalid_directories: Vec<String>,
    #[serde(default)]
    pub placeholder_tokens: Vec<String>,
    #[serde(default)]
    pub runtime_checked: bool,
    #[serde(default = "default_runtime_ok")]
    pub runtime_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_note: Option<String>,
}

fn default_runtime_ok() -> bool {
    true
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.syntax_ok
            && self.missing_binaries.is_empty()
            && self.alias_conflicts.is_empty()
            && self.invalid_directories.is_empty()
            && self.placeholder_tokens.is_empty()
            && (!self.runtime_checked || self.runtime_ok)
    }

    pub fn to_feedback_text(&self) -> String {
        let mut reasons = Vec::new();
        if !self.syntax_ok {
            reasons.push(format!("shell syntax check failed by {}", self.shell));
        }
        if !self.missing_binaries.is_empty() {
            reasons.push(format!(
                "unresolved commands: {}",
                self.missing_binaries.join(", ")
            ));
        }
        if !self.alias_conflicts.is_empty() {
            reasons.push(format!(
                "alias command(s) detected: {} (prefix with builtin or \\\\)",
                self.alias_conflicts.join(", ")
            ));
        }
        if !self.invalid_directories.is_empty() {
            reasons.push(format!(
                "directory not found for cd: {}",
                self.invalid_directories.join(", ")
            ));
        }
        if !self.placeholder_tokens.is_empty() {
            reasons.push(format!(
                "placeholder tokens are not allowed: {}",
                self.placeholder_tokens.join(", ")
            ));
        }
        if self.runtime_checked && !self.runtime_ok {
            if let Some(note) = &self.runtime_note {
                reasons.push(format!("runtime validation failed: {}", note));
            } else {
                reasons.push("runtime validation failed".to_string());
            }
        }
        if reasons.is_empty() {
            "validation failed for an unknown reason".to_string()
        } else {
            reasons.join("; ")
        }
    }
}
