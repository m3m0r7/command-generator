use super::split::split_top_level_segment_ranges;
use super::words::{is_echo_word, next_word_bounds};

pub(super) fn normalize_echo_default(command: &str) -> String {
    let ranges = split_top_level_segment_ranges(command);
    if ranges.is_empty() {
        return command.trim().to_string();
    }

    let mut out = String::new();
    let mut cursor = 0usize;
    for (start, end) in ranges {
        if cursor < start {
            out.push_str(&command[cursor..start]);
        }
        out.push_str(&normalize_echo_segment(&command[start..end]));
        cursor = end;
    }
    if cursor < command.len() {
        out.push_str(&command[cursor..]);
    }
    out.trim().to_string()
}

fn normalize_echo_segment(segment: &str) -> String {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return segment.to_string();
    }

    let leading = segment.len() - segment.trim_start().len();
    let trailing = segment.len() - segment.trim_end().len();
    let core_end = segment.len().saturating_sub(trailing);
    let core = &segment[leading..core_end];

    let Some(first) = next_word_bounds(core, 0) else {
        return segment.to_string();
    };
    let first_word = &core[first.0..first.1];

    let (_, after_echo) = if first_word == "builtin" || first_word == "command" {
        let Some(second) = next_word_bounds(core, first.1) else {
            return segment.to_string();
        };
        let second_word = &core[second.0..second.1];
        if !is_echo_word(second_word) {
            return segment.to_string();
        }
        (second, second.1)
    } else {
        if !is_echo_word(first_word) {
            return segment.to_string();
        }
        (first, first.1)
    };

    let Some(arg) = next_word_bounds(core, after_echo) else {
        let mut with_flag = String::new();
        with_flag.push_str(&segment[..leading]);
        with_flag.push_str(core);
        with_flag.push_str(" -e");
        with_flag.push_str(&segment[core_end..]);
        return with_flag;
    };
    let arg_word = &core[arg.0..arg.1];
    if arg_word.starts_with('-') {
        return segment.to_string();
    }

    let mut with_flag = String::new();
    with_flag.push_str(&segment[..leading]);
    with_flag.push_str(&core[..arg.0]);
    with_flag.push_str("-e ");
    with_flag.push_str(&core[arg.0..]);
    with_flag.push_str(&segment[core_end..]);
    with_flag
}
