mod head;
mod tokenize;

use super::segments::split_segments;
use crate::validation::types::{CommandHead, SegmentToken};

pub(crate) fn collect_command_heads(command: &str) -> Vec<CommandHead> {
    split_segments(command)
        .into_iter()
        .filter_map(|segment| extract_head_command(&segment))
        .collect::<Vec<_>>()
}

pub(crate) fn extract_head_command(segment: &str) -> Option<CommandHead> {
    let tokens = tokenize::tokenize_segment(segment);
    head::locate_head_token(&tokens)
}

pub(crate) fn tokenize_segment(segment: &str) -> Vec<SegmentToken> {
    tokenize::tokenize_segment(segment)
}

pub(crate) fn locate_head_token(tokens: &[SegmentToken]) -> Option<CommandHead> {
    head::locate_head_token(tokens)
}
