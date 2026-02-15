use anyhow::{Result, anyhow};
use rustyline::error::ReadlineError;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Copy)]
pub enum ClarificationKind {
    YesNo,
    Text,
}

pub trait ClarificationPrompter {
    fn ask(&mut self, kind: ClarificationKind, question: &str) -> Result<String>;
}

pub struct EditorPrompter<'a> {
    editor: &'a mut rustyline::DefaultEditor,
}

impl<'a> EditorPrompter<'a> {
    pub fn new(editor: &'a mut rustyline::DefaultEditor) -> Self {
        Self { editor }
    }
}

impl ClarificationPrompter for EditorPrompter<'_> {
    fn ask(&mut self, kind: ClarificationKind, question: &str) -> Result<String> {
        match kind {
            ClarificationKind::YesNo => ask_yes_no_with_editor(self.editor, question),
            ClarificationKind::Text => ask_text_with_editor(self.editor, question),
        }
    }
}

pub struct StdioPrompter;

impl StdioPrompter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdioPrompter {
    fn default() -> Self {
        Self::new()
    }
}

impl ClarificationPrompter for StdioPrompter {
    fn ask(&mut self, kind: ClarificationKind, question: &str) -> Result<String> {
        match kind {
            ClarificationKind::YesNo => ask_yes_no_with_stdio(question),
            ClarificationKind::Text => ask_text_with_stdio(question),
        }
    }
}

fn ask_yes_no_with_editor(editor: &mut rustyline::DefaultEditor, question: &str) -> Result<String> {
    loop {
        let prompt = format!("? {} [y/n]: ", question.trim());
        match editor.readline(&prompt) {
            Ok(line) => {
                if let Some(answer) = normalize_yes_no_answer(&line) {
                    let _ = editor.add_history_entry(line.trim());
                    return Ok(answer.to_string());
                }
                eprintln!("please answer with y or n");
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                return Err(anyhow!("clarification aborted"));
            }
            Err(err) => return Err(err.into()),
        }
    }
}

fn ask_text_with_editor(editor: &mut rustyline::DefaultEditor, question: &str) -> Result<String> {
    loop {
        let prompt = format!("? {}: ", question.trim());
        match editor.readline(&prompt) {
            Ok(line) => {
                let answer = line.trim();
                if !answer.is_empty() {
                    let _ = editor.add_history_entry(answer);
                    return Ok(answer.to_string());
                }
                eprintln!("please input a non-empty value");
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                return Err(anyhow!("clarification aborted"));
            }
            Err(err) => return Err(err.into()),
        }
    }
}

fn ask_yes_no_with_stdio(question: &str) -> Result<String> {
    let stdin = io::stdin();
    let mut lock = stdin.lock();
    let mut line = String::new();
    loop {
        line.clear();
        print!("? {} [y/n]: ", question.trim());
        io::stdout().flush()?;
        if lock.read_line(&mut line)? == 0 {
            return Err(anyhow!("clarification aborted"));
        }
        if let Some(answer) = normalize_yes_no_answer(&line) {
            return Ok(answer.to_string());
        }
        eprintln!("please answer with y or n");
    }
}

fn ask_text_with_stdio(question: &str) -> Result<String> {
    let stdin = io::stdin();
    let mut lock = stdin.lock();
    let mut line = String::new();
    loop {
        line.clear();
        print!("? {}: ", question.trim());
        io::stdout().flush()?;
        if lock.read_line(&mut line)? == 0 {
            return Err(anyhow!("clarification aborted"));
        }
        let answer = line.trim();
        if !answer.is_empty() {
            return Ok(answer.to_string());
        }
        eprintln!("please input a non-empty value");
    }
}

fn normalize_yes_no_answer(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_lowercase();
    match normalized.as_str() {
        "y" | "yes" | "true" | "1" => Some("yes"),
        "n" | "no" | "false" | "0" => Some("no"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_yes_no_answer() {
        assert_eq!(normalize_yes_no_answer("Y"), Some("yes"));
        assert_eq!(normalize_yes_no_answer(" no "), Some("no"));
        assert_eq!(normalize_yes_no_answer("maybe"), None);
    }
}
