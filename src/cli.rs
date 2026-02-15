use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "command-generator",
    version,
    about = "LLM-powered interactive command generator"
)]
pub struct Cli {
    /// Model name or provider:model (e.g. openai:gpt-5.2)
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// API key (overrides environment variable)
    #[arg(short = 'k', long = "key")]
    pub key: Option<String>,

    /// Show cached/fetched model list and exit
    #[arg(long = "show-models-list")]
    pub show_models_list: bool,

    /// Copy generated command to clipboard
    #[arg(short = 'c', long = "copy")]
    pub copy: bool,

    /// Resume a previous session by UUID
    #[arg(short = 'r', long = "resume")]
    pub resume: Option<String>,

    /// Run once in non-interactive mode
    #[arg(long = "once")]
    pub once: Option<String>,

    /// Number of shell history lines to include in prompt context
    #[arg(long = "history-lines", default_value_t = 80)]
    pub history_lines: usize,

    /// Number of generated command lines to include from previous sessions
    #[arg(long = "generated-history-lines", default_value_t = 80)]
    pub generated_history_lines: usize,

    /// Number of in-session turns to include in prompt context
    #[arg(long = "context-turns", default_value_t = 12)]
    pub context_turns: usize,

    /// Validation retry count
    #[arg(long = "max-attempts", default_value_t = 3)]
    pub max_attempts: usize,

    /// Print explanation blocks under generated command
    #[arg(short = 'e', long = "explanation")]
    pub explanation: bool,
}
