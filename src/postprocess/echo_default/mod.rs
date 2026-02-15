mod normalize;
mod split;
mod words;

use anyhow::Result;

use crate::postprocess::CommandPostProcessor;

pub struct EchoDefaultStage;

impl CommandPostProcessor for EchoDefaultStage {
    fn process(&self, _shell: &str, command: String) -> Result<String> {
        Ok(normalize::normalize_echo_default(&command))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize::normalize_echo_default;

    #[test]
    fn defaults_echo_to_dash_e() {
        assert_eq!(normalize_echo_default("echo hello"), "echo -e hello");
        assert_eq!(normalize_echo_default("echo -n hello"), "echo -n hello");
        assert_eq!(
            normalize_echo_default("pwd && echo hello"),
            "pwd && echo -e hello"
        );
        assert_eq!(
            normalize_echo_default("builtin echo hello"),
            "builtin echo -e hello"
        );
    }
}
