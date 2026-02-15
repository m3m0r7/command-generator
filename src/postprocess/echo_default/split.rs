pub(super) fn split_top_level_segment_ranges(command: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut current_start = 0usize;
    let mut chars = command.char_indices().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    while let Some((idx, ch)) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && !in_single {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if in_single || in_double {
            continue;
        }

        if ch == ';' {
            push_top_level_range(&mut ranges, command, current_start, idx);
            current_start = idx + ch.len_utf8();
            continue;
        }
        if ch == '|' {
            if let Some((next_idx, next_ch)) = chars.peek().copied()
                && next_ch == '|'
            {
                let _ = chars.next();
                push_top_level_range(&mut ranges, command, current_start, idx);
                current_start = next_idx + next_ch.len_utf8();
                continue;
            }
            push_top_level_range(&mut ranges, command, current_start, idx);
            current_start = idx + ch.len_utf8();
            continue;
        }
        if ch == '&'
            && let Some((next_idx, next_ch)) = chars.peek().copied()
            && next_ch == '&'
        {
            let _ = chars.next();
            push_top_level_range(&mut ranges, command, current_start, idx);
            current_start = next_idx + next_ch.len_utf8();
            continue;
        }
    }
    push_top_level_range(&mut ranges, command, current_start, command.len());
    ranges
}

fn push_top_level_range(ranges: &mut Vec<(usize, usize)>, command: &str, start: usize, end: usize) {
    if start >= end {
        return;
    }
    if let Some(slice) = command.get(start..end)
        && !slice.trim().is_empty()
    {
        ranges.push((start, end));
    }
}
