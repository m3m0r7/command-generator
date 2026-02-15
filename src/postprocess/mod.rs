mod alias_prefix;
mod and_or;
mod echo_default;

use anyhow::Result;

pub use alias_prefix::AliasPrefixStage;
pub use and_or::AndOrPrecedenceStage;
pub use echo_default::EchoDefaultStage;

pub trait CommandPostProcessor: Send + Sync {
    fn process(&self, shell: &str, command: String) -> Result<String>;
}

pub struct PostProcessPipeline {
    stages: Vec<Box<dyn CommandPostProcessor>>,
}

impl PostProcessPipeline {
    pub fn new(stages: Vec<Box<dyn CommandPostProcessor>>) -> Self {
        Self { stages }
    }

    pub fn with_default_stages() -> Self {
        Self::new(vec![
            Box::new(AndOrPrecedenceStage),
            Box::new(EchoDefaultStage),
            Box::new(AliasPrefixStage),
        ])
    }
}

impl CommandPostProcessor for PostProcessPipeline {
    fn process(&self, shell: &str, mut command: String) -> Result<String> {
        for stage in &self.stages {
            command = stage.process(shell, command)?;
        }
        Ok(command)
    }
}

pub fn default_post_processor() -> Box<dyn CommandPostProcessor> {
    Box::new(PostProcessPipeline::with_default_stages())
}
