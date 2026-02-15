use anyhow::Result;
use rustyline::error::ReadlineError;

use crate::cli::Cli;
use crate::interactive::is_exit_command;
use crate::output;
use crate::prompter::EditorPrompter;
use crate::request_engine::RequestEngine;
use crate::session::SessionRecord;

pub async fn run(cli: &Cli, engine: &RequestEngine<'_>, session: &mut SessionRecord) -> Result<()> {
    let mut editor = rustyline::DefaultEditor::new()?;
    loop {
        match editor.readline("> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = editor.add_history_entry(input);
                if is_exit_command(input) {
                    println!("Good Bye!");
                    break;
                }

                let mut prompter = EditorPrompter::new(&mut editor);
                match engine.generate(input, session, Some(&mut prompter)).await {
                    Ok(result) => output::print_generated_result(&result, cli.explanation)?,
                    Err(err) => eprintln!("error: {err}"),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Good Bye!");
                break;
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}
