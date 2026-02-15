pub(crate) fn find_placeholder_tokens(command: &str) -> Vec<String> {
    let banned_words = ["YOUR_VALUE", "REPLACE_ME", "INSERT_VALUE", "PLACEHOLDER"];
    let mut found = Vec::new();
    let lower = command.to_lowercase();
    for word in banned_words {
        if lower.contains(&word.to_lowercase()) {
            found.push(word.to_string());
        }
    }

    for token in shell_words::split(command).unwrap_or_default() {
        if token.len() >= 3
            && token.starts_with('<')
            && token.ends_with('>')
            && !token.contains('/')
            && !token.contains('\\')
        {
            found.push(token);
        }
    }
    found.sort();
    found.dedup();
    found
}
