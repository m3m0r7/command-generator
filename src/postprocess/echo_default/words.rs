pub(super) fn next_word_bounds(input: &str, from: usize) -> Option<(usize, usize)> {
    let mut start = None;
    let mut end = input.len();
    for (idx, ch) in input[from..].char_indices() {
        let absolute = from + idx;
        if start.is_none() {
            if ch.is_whitespace() {
                continue;
            }
            start = Some(absolute);
            continue;
        }
        if ch.is_whitespace() {
            end = absolute;
            break;
        }
    }
    let start = start?;
    Some((start, end))
}

pub(super) fn is_echo_word(word: &str) -> bool {
    word == "echo" || word == "\\echo"
}
