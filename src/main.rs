mod cli;
mod clipboard;
mod history;
mod llm;
mod meta;
mod model;
mod paths;
mod prompt;
mod session;
mod validation;

use anyhow::{Result, anyhow};
use clap::Parser;
use rustyline::error::ReadlineError;
use std::io::{self, BufRead, IsTerminal, Write};

use cli::Cli;
use llm::{LlmClient, LlmOutput};
use model::{ProviderKind, ProviderSelection};
use prompt::{PromptClarification, PromptInput, PromptTurn};
use session::SessionRecord;

#[derive(Debug, Clone, Copy)]
enum PromptAnswerKind {
    YesNo,
    Text,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    paths::ensure_dirs()?;

    let resumed_session = match cli.resume.as_deref() {
        Some(uuid) => Some(session::load_session(uuid)?),
        None => None,
    };

    if cli.show_models_list {
        let provider =
            resolve_provider_for_model_listing(cli.model.as_deref(), resumed_session.as_ref())?;
        let key = cli
            .key
            .clone()
            .or_else(|| model::resolve_key(provider, None).ok());
        let models = meta::get_models(provider, key.as_deref()).await?;
        for model in models {
            println!("{model}");
        }
        return Ok(());
    }

    let provider = resolve_provider(
        cli.model.as_deref(),
        cli.key.as_deref(),
        resumed_session.as_ref(),
    )?;
    let model_name = resolve_model_name(provider, cli.model.as_deref(), resumed_session.as_ref())?;
    let api_key = model::resolve_key(provider, cli.key.as_deref())?;

    meta::set_last_using_model(provider, &model_name)?;

    let llm = LlmClient::new(provider, api_key, model_name.clone());
    let mut active_session =
        resumed_session.unwrap_or_else(|| SessionRecord::new(provider, &model_name));
    active_session.provider = provider.as_str().to_string();
    active_session.model = model_name.clone();
    session::save_session(&active_session)?;
    eprintln!(
        "Session UUID: {} (resume with: command-generator --resume {})",
        active_session.uuid, active_session.uuid
    );

    if let Some(request) = cli.once.as_deref() {
        let command = handle_request(request, &cli, &llm, &mut active_session, None).await?;
        println!("{command}");
        return Ok(());
    }

    run_interactive(&cli, &llm, &mut active_session).await
}

fn resolve_provider_for_model_listing(
    model_arg: Option<&str>,
    resumed_session: Option<&SessionRecord>,
) -> Result<ProviderKind> {
    if let Some(model_arg) = model_arg {
        let selection = model::resolve_provider_selection(Some(model_arg), None, true)?;
        return Ok(selection.provider);
    }
    if let Some(session) = resumed_session
        && let Some(provider) = model::provider_from_name(&session.provider)
    {
        return Ok(provider);
    }
    let selection = model::resolve_provider_selection(None, None, true)?;
    Ok(selection.provider)
}

fn resolve_provider(
    model_arg: Option<&str>,
    key_arg: Option<&str>,
    resumed_session: Option<&SessionRecord>,
) -> Result<ProviderKind> {
    if let Some(model_arg) = model_arg {
        let selection = model::resolve_provider_selection(Some(model_arg), key_arg, false)?;
        return Ok(selection.provider);
    }
    if let Some(session) = resumed_session
        && let Some(provider) = model::provider_from_name(&session.provider)
    {
        return Ok(provider);
    }
    let selection = model::resolve_provider_selection(None, key_arg, false)?;
    Ok(selection.provider)
}

fn resolve_model_name(
    provider: ProviderKind,
    model_arg: Option<&str>,
    resumed_session: Option<&SessionRecord>,
) -> Result<String> {
    if let Some(model_arg) = model_arg {
        let ProviderSelection {
            requested_model, ..
        } = model::resolve_provider_selection(Some(model_arg), None, true)?;
        return Ok(requested_model.unwrap_or_else(|| model::default_model(provider).to_string()));
    }

    if let Some(session) = resumed_session {
        let value = session.model.trim();
        if !value.is_empty() {
            return Ok(value.to_string());
        }
    }

    if let Some(last) = meta::get_last_using_model(provider)? {
        return Ok(last);
    }
    Ok(model::default_model(provider).to_string())
}

async fn run_interactive(cli: &Cli, llm: &LlmClient, session: &mut SessionRecord) -> Result<()> {
    if cli.resume.is_some() {
        print_resumed_context(session, cli.context_turns);
    }
    println!("Interactive mode. Type exit to finish.");
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        let mut editor = rustyline::DefaultEditor::new()?;
        loop {
            match editor.readline("> ") {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }
                    let _ = editor.add_history_entry(input);
                    if matches!(input, "exit" | "quit" | "/exit" | "/quit") {
                        println!("Good Bye!");
                        break;
                    }

                    let mut ask = |kind: PromptAnswerKind, question: &str| match kind {
                        PromptAnswerKind::YesNo => ask_yes_no_with_editor(&mut editor, question),
                        PromptAnswerKind::Text => ask_text_with_editor(&mut editor, question),
                    };
                    match handle_request(input, cli, llm, session, Some(&mut ask)).await {
                        Ok(command) => println!("{command}"),
                        Err(err) => eprintln!("error: {err}"),
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("Good Bye!");
                    break;
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }
        return Ok(());
    }

    let stdin = io::stdin();
    let mut lock = stdin.lock();
    let mut line = String::new();
    loop {
        line.clear();
        print!("> ");
        io::stdout().flush()?;
        if lock.read_line(&mut line)? == 0 {
            println!("Good Bye!");
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if matches!(input, "exit" | "quit" | "/exit" | "/quit") {
            println!("Good Bye!");
            break;
        }
        let mut ask = |kind: PromptAnswerKind, question: &str| match kind {
            PromptAnswerKind::YesNo => ask_yes_no_with_stdio(question),
            PromptAnswerKind::Text => ask_text_with_stdio(question),
        };
        match handle_request(input, cli, llm, session, Some(&mut ask)).await {
            Ok(command) => println!("{command}"),
            Err(err) => eprintln!("error: {err}"),
        }
    }

    Ok(())
}

fn print_resumed_context(session: &SessionRecord, limit: usize) {
    if session.turns.is_empty() {
        println!("Resumed session has no prior turns.");
        return;
    }
    let recent = session.recent_turns(limit.max(1));
    println!(
        "Resumed context (showing {} turn(s) of {}):",
        recent.len(),
        session.turns.len()
    );
    for turn in recent {
        println!("> {}", turn.user_input);
        println!("{}", turn.command);
    }
    println!("---");
}

async fn handle_request(
    user_input: &str,
    cli: &Cli,
    llm: &LlmClient,
    session: &mut SessionRecord,
    mut ask_user: Option<&mut dyn FnMut(PromptAnswerKind, &str) -> Result<String>>,
) -> Result<String> {
    let shell_history = history::load_shell_history(cli.history_lines);
    let generated_history = session::list_recent_commands(cli.generated_history_lines)?;
    let turns = session
        .recent_turns(cli.context_turns)
        .into_iter()
        .map(|turn| PromptTurn {
            user_input: turn.user_input,
            command: turn.command,
        })
        .collect::<Vec<_>>();

    let os = std::env::consts::OS.to_string();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    let mut clarifications = Vec::new();
    let mut asked_questions = std::collections::HashSet::new();
    let mut feedback = None;
    let max_attempts = cli.max_attempts.max(1);
    let max_questions = 8usize;
    let mut question_count = 0usize;
    let mut command_attempt_count = 0usize;
    let mut last_error = None;

    while command_attempt_count < max_attempts {
        let prompt = prompt::render(&PromptInput {
            os: os.clone(),
            shell: shell.clone(),
            session_uuid: session.uuid.clone(),
            model: llm.model_name().to_string(),
            command_tool_name: llm::COMMAND_TOOL_NAME.to_string(),
            question_tool_name: llm::QUESTION_TOOL_NAME.to_string(),
            text_question_tool_name: llm::TEXT_QUESTION_TOOL_NAME.to_string(),
            user_input: user_input.to_string(),
            shell_history: shell_history.clone(),
            generated_history: generated_history.clone(),
            turns: turns.clone(),
            clarifications: clarifications.clone(),
            feedback: feedback.clone(),
        })?;

        match llm.generate_output(&prompt).await? {
            LlmOutput::Command(candidate) => {
                if ask_user.is_some()
                    && clarifications.is_empty()
                    && has_runtime_input_prompt(&candidate.command)
                {
                    let reason = "Do not use runtime read prompts in the final command. Ask a text clarification question first via tool.".to_string();
                    feedback = Some(reason.clone());
                    last_error = Some(anyhow!(reason));
                    continue;
                }
                command_attempt_count += 1;
                let report = validation::validate_command(&candidate.command)?;
                if report.is_valid() {
                    let command = candidate.command.trim().to_string();
                    if cli.copy
                        && let Err(err) = clipboard::copy_text(&command)
                    {
                        eprintln!("warning: failed to copy command: {err}");
                    }
                    session.push_turn(user_input, command.clone(), candidate.reason, report);
                    session::save_session(session)?;
                    return Ok(command);
                }

                let reason = report.to_feedback_text();
                feedback = Some(reason.clone());
                last_error = Some(anyhow!(reason));
            }
            LlmOutput::QuestionYesNo(question) => {
                question_count += 1;
                if question_count > max_questions {
                    return Err(anyhow!("too many clarification questions from model"));
                }
                let normalized_question = normalize_question_text(&question.question);
                if !asked_questions.insert(normalized_question.clone()) {
                    return Err(anyhow!(
                        "model asked duplicate clarification question: {}",
                        question.question
                    ));
                }
                let answer = match ask_user.as_mut() {
                    Some(asker) => asker(PromptAnswerKind::YesNo, &question.question)?,
                    None => {
                        return Err(anyhow!(
                            "model requested clarification ('{}') but --once mode cannot answer y/n; run interactive mode",
                            question.question
                        ));
                    }
                };
                clarifications.push(PromptClarification {
                    question: question.question,
                    answer,
                });
                feedback = None;
            }
            LlmOutput::QuestionText(question) => {
                question_count += 1;
                if question_count > max_questions {
                    return Err(anyhow!("too many clarification questions from model"));
                }
                let normalized_question = normalize_question_text(&question.question);
                if !asked_questions.insert(normalized_question.clone()) {
                    return Err(anyhow!(
                        "model asked duplicate clarification question: {}",
                        question.question
                    ));
                }
                let answer = match ask_user.as_mut() {
                    Some(asker) => asker(PromptAnswerKind::Text, &question.question)?,
                    None => {
                        return Err(anyhow!(
                            "model requested clarification ('{}') but --once mode cannot answer text; run interactive mode",
                            question.question
                        ));
                    }
                };
                clarifications.push(PromptClarification {
                    question: question.question,
                    answer,
                });
                feedback = None;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("failed to generate a valid command")))
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

fn has_runtime_input_prompt(command: &str) -> bool {
    let lowered = command.to_lowercase();
    lowered.contains("read ")
        || lowered.contains("read\t")
        || lowered.contains("vared ")
        || lowered.contains("vared\t")
}

fn normalize_yes_no_answer(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_lowercase();
    match normalized.as_str() {
        "y" | "yes" | "true" | "1" => Some("yes"),
        "n" | "no" | "false" | "0" => Some("no"),
        _ => None,
    }
}

fn normalize_question_text(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn normalizes_question_text_for_duplicate_detection() {
        assert_eq!(
            normalize_question_text("  Use recursive  search? "),
            "use recursive search?"
        );
    }

    #[test]
    fn detects_runtime_input_prompt() {
        assert!(has_runtime_input_prompt("read -r x"));
        assert!(has_runtime_input_prompt("vared target"));
        assert!(!has_runtime_input_prompt("echo hello"));
    }
}
