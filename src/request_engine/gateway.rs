use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

use crate::llm::{LlmClient, LlmOutput};

pub trait GenerationGateway: Send + Sync {
    fn model_name(&self) -> &str;

    fn generate_output<'a>(
        &'a self,
        system_prompt: &'a str,
        user_prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmOutput>> + Send + 'a>>;
}

impl GenerationGateway for LlmClient {
    fn model_name(&self) -> &str {
        self.model_name()
    }

    fn generate_output<'a>(
        &'a self,
        system_prompt: &'a str,
        user_prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmOutput>> + Send + 'a>> {
        Box::pin(self.generate_output(system_prompt, user_prompt))
    }
}
