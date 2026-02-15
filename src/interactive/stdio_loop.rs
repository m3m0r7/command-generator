use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::cli::Cli;
use crate::interactive::is_exit_command;
use crate::output;
use crate::prompter::StdioPrompter;
use crate::request_engine::RequestEngine;
use crate::session::SessionRecord;

pub async fn run(cli: &Cli, engine: &RequestEngine<'_>, session: &mut SessionRecord) -> Result<()> {
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
        if is_exit_command(input) {
            println!("Good Bye!");
            break;
        }

        let mut prompter = StdioPrompter::new();
        match engine.generate(input, session, Some(&mut prompter)).await {
            Ok(result) => output::print_generated_result(&result, cli.explanation)?,
            Err(err) => eprintln!("error: {err}"),
        }
    }
    Ok(())
}
