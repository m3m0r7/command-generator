mod stdio_loop;
mod tty_loop;

use anyhow::Result;
use std::future::Future;
use std::io::{self, IsTerminal};
use std::pin::Pin;

use crate::cli::Cli;
use crate::request_engine::RequestEngine;
use crate::session::SessionRecord;

trait InteractiveBackend {
    fn run<'a>(
        &'a self,
        cli: &'a Cli,
        engine: &'a RequestEngine<'a>,
        session: &'a mut SessionRecord,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}

struct TtyBackend;
struct StdioBackend;

impl InteractiveBackend for TtyBackend {
    fn run<'a>(
        &'a self,
        cli: &'a Cli,
        engine: &'a RequestEngine<'a>,
        session: &'a mut SessionRecord,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(tty_loop::run(cli, engine, session))
    }
}

impl InteractiveBackend for StdioBackend {
    fn run<'a>(
        &'a self,
        cli: &'a Cli,
        engine: &'a RequestEngine<'a>,
        session: &'a mut SessionRecord,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(stdio_loop::run(cli, engine, session))
    }
}

pub async fn run_interactive(
    cli: &Cli,
    engine: &RequestEngine<'_>,
    session: &mut SessionRecord,
) -> Result<()> {
    if cli.resume.is_some() {
        print_resumed_context(session, cli.context_turns);
    }
    println!("Interactive mode. Type exit to finish.");

    let backend: &dyn InteractiveBackend =
        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            &TtyBackend
        } else {
            &StdioBackend
        };
    backend.run(cli, engine, session).await
}

pub fn is_exit_command(input: &str) -> bool {
    matches!(input, "exit" | "quit" | "/exit" | "/quit")
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
