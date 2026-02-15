use super::parser::{collect_command_heads, extract_head_command, find_invalid_cd_directories};
use super::runtime::can_runtime_check;
use super::*;

#[test]
fn splits_compound_command() {
    let heads = collect_command_heads("pwd || true; ls -la | grep src");
    let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
    assert_eq!(names, vec!["pwd", "true", "ls", "grep"]);
}

#[test]
fn handles_no_space_pipeline() {
    let heads = collect_command_heads("cat Cargo.toml|grep name");
    let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
    assert_eq!(names, vec!["cat", "grep"]);
}

#[test]
fn skips_env_assignment() {
    let heads = collect_command_heads("FOO=bar env ls");
    let names = heads.into_iter().map(|head| head.name).collect::<Vec<_>>();
    assert_eq!(names, vec!["ls"]);
}

#[test]
fn allows_runtime_for_simple_readonly_command() {
    let heads = collect_command_heads("pwd");
    assert!(can_runtime_check("pwd", &heads));
}

#[test]
fn skips_runtime_for_risky_command() {
    let heads = collect_command_heads("rm -rf /tmp/foo");
    assert!(!can_runtime_check("rm -rf /tmp/foo", &heads));
}

#[test]
fn detects_placeholder_tokens() {
    let tokens = parser::find_placeholder_tokens("echo <STRING>");
    assert_eq!(tokens, vec!["<STRING>"]);
}

#[test]
fn detects_builtin_prefix() {
    let head = extract_head_command("builtin test -f Cargo.toml").unwrap();
    assert_eq!(head.name, "test");
    assert!(head.prefixed_builtin);
}

#[test]
fn detects_command_prefix() {
    let head = extract_head_command("command ls -la").unwrap();
    assert_eq!(head.name, "ls");
    assert!(head.prefixed_command);
}

#[test]
fn detects_backslash_prefix() {
    let head = extract_head_command("\\ls -la").unwrap();
    assert_eq!(head.name, "ls");
    assert!(head.prefixed_backslash);
}

#[test]
fn detects_missing_cd_directory() {
    let invalid = find_invalid_cd_directories("cd ./this_should_not_exist_12345 && pwd");
    assert_eq!(invalid, vec!["./this_should_not_exist_12345"]);
}

#[test]
fn leaves_prefixed_command_unchanged() {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    let command = "builtin pwd";
    assert_eq!(
        normalize_alias_prefixes(&shell, command).unwrap(),
        "builtin pwd"
    );
}
