# Command Generator

`command-generator` is a Rust CLI tool that generates shell commands interactively using an LLM.  
It includes command validation, session persistence/resume, and clarification questions for missing inputs.

## Features

- Interactive command generation (default mode)
- OpenAI / Gemini / Claude support
- Function-calling workflow with 3 tools
  - `deliver_command`: returns the final command
  - `ask_yes_no_question`: asks a yes/no clarification
  - `ask_text_question`: asks a free-text clarification
- Validation pipeline
  - shell syntax check (`$SHELL -n -c`)
  - command resolution check (`which` + `command -v`)
  - alias conflict detection (prompts `builtin` / `command` / `\` prefix)
  - placeholder rejection (`<STRING>`, etc.)
  - runtime smoke test in `/tmp` for safe commands only
- Session save/resume by UUID
- Prints prior context when started with `--resume`
- `-c/--copy` to copy generated command to clipboard

## Requirements

- Rust (stable)
- At least one API key
  - OpenAI: `OPENAI_API_KEY`
  - Gemini: `GEMINI_API_KEY` or `GOOGLE_API_KEY`
  - Claude: `ANTHROPIC_API_KEY`

## Installation

```bash
cargo install --path .
```

Optional alias:

```bash
alias cg="command-generator"
```

## Quick Start

```bash
cg
```

Example:

```text
> Output yes if $PATH contains a specific string, otherwise no
? Enter the string to check in PATH: mytool
[[ ":$PATH:" == *":mytool:"* ]] && print -r -- yes || print -r -- no
> exit
Good Bye!
```

## Conversation Examples

### Example 1: Missing value resolved by text clarification

```text
> Output yes if $PATH contains a specific string, otherwise no
? Enter the string to check in PATH: mytool
[[ ":$PATH:" == *":mytool:"* ]] && print -r -- yes || print -r -- no
```

### Example 2: Requirement fixed by yes/no clarification

```text
> Search recursively for *.rs files
? Include hidden directories (such as .git)? [y/n]: n
find . -type f -name '*.rs' -not -path '*/.*/*'
```

### Example 3: Resume prints prior context

```text
$ cg --resume e13d0964-7710-41e8-a7bc-f5d197b7c1f7
Resumed context (showing 1 turn(s) of 1):
> output pwd
pwd
---
Interactive mode. Type exit to finish.
```

## CLI Options

```text
-m, --model <MODEL>                       Model name or provider:model
-k, --key <KEY>                           API key (overrides env var)
    --show-models-list                    Show model list
-c, --copy                                Copy generated command
-r, --resume <UUID>                       Resume session
    --once <REQUEST>                      Run once in non-interactive mode
    --history-lines <N>                   Shell history lines (default: 80)
    --generated-history-lines <N>         Generated-command history lines (default: 80)
    --context-turns <N>                   In-session context turns (default: 12)
    --max-attempts <N>                    Regeneration attempts after validation failure (default: 3)
```

## Models and Providers

### Model selection examples

- `-m openai:gpt-5.2`
- `-m gemini:gemini-2.5-flash`
- `-m claude:claude-sonnet-4-5`
- Provider-only form is also supported (e.g. `-m openai`)

### Default provider resolution

Automatically selected by available API keys in this order:

1. `OPENAI_API_KEY`
2. `GEMINI_API_KEY` / `GOOGLE_API_KEY`
3. `ANTHROPIC_API_KEY`

### Model list

```bash
cg --show-models-list
cg --show-models-list -m gemini
```

Model lists are cached with a TTL of 24 hours.

## Session Persistence and Resume

Each generation is saved in a UUID-based session.

```bash
cg --resume <uuid>
# or
cg -r <uuid>
```

Default paths:

- `~/.command-generator/sessions/*.json`
- `~/.command-generator/.cache/meta.json`

Set `COMMAND_GENERATOR_DIR` to override the storage root.

## Validation Policy

Generated commands are validated with:

1. shell syntax check
2. command resolvability
3. alias conflict check (enforces `builtin`, `command`, or `\` when needed)
4. placeholder rejection
5. runtime smoke test for safe commands only

Note: `--once` cannot answer clarification questions.  
If the model needs clarification, run in interactive mode (`cg`).

## Development Commands

```bash
make build
make test
make fmt
make clippy
make release
make install
make command-generator-build
```

## License

See `LICENSE`.
