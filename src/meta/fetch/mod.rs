mod claude;
mod error;
mod gemini;
mod openai;

use anyhow::Result;

use crate::model::ProviderKind;

pub(crate) async fn fetch_models(provider: ProviderKind, key: &str) -> Result<Vec<String>> {
    match provider {
        ProviderKind::OpenAI => openai::fetch_models_openai(key).await,
        ProviderKind::Gemini => gemini::fetch_models_gemini(key).await,
        ProviderKind::Claude => claude::fetch_models_claude(key).await,
    }
}
