mod cd_checks;
mod placeholders;
mod segments;
mod tokens;

pub(crate) use cd_checks::find_invalid_cd_directories;
pub(crate) use placeholders::find_placeholder_tokens;
pub(crate) use segments::split_segment_ranges;
pub(crate) use tokens::{collect_command_heads, locate_head_token, tokenize_segment};

#[cfg(test)]
pub(crate) use tokens::extract_head_command;
