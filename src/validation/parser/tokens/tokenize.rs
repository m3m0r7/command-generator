use crate::validation::types::SegmentToken;

pub(super) fn tokenize_segment(segment: &str) -> Vec<SegmentToken> {
    let mut tokens = Vec::new();
    let mut raw = String::new();
    let mut cooked = String::new();
    let mut in_token = false;
    let mut token_start = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    let push_token = |tokens: &mut Vec<SegmentToken>,
                      raw: &mut String,
                      cooked: &mut String,
                      in_token: &mut bool,
                      start: usize| {
        if !*in_token {
            return;
        }
        tokens.push(SegmentToken {
            raw: raw.clone(),
            cooked: cooked.clone(),
            start,
        });
        raw.clear();
        cooked.clear();
        *in_token = false;
    };

    for (idx, ch) in segment.char_indices() {
        if !in_token {
            if ch.is_whitespace() {
                continue;
            }
            in_token = true;
            token_start = idx;
        }

        if escaped {
            raw.push(ch);
            cooked.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' && !in_single {
            raw.push(ch);
            escaped = true;
            continue;
        }

        if ch == '\'' && !in_double {
            raw.push(ch);
            in_single = !in_single;
            continue;
        }

        if ch == '"' && !in_single {
            raw.push(ch);
            in_double = !in_double;
            continue;
        }

        if !in_single && !in_double && ch.is_whitespace() {
            push_token(
                &mut tokens,
                &mut raw,
                &mut cooked,
                &mut in_token,
                token_start,
            );
            continue;
        }

        raw.push(ch);
        cooked.push(ch);
    }

    if in_token {
        tokens.push(SegmentToken {
            raw,
            cooked,
            start: token_start,
        });
    }

    tokens
}
