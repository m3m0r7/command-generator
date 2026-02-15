use crate::llm::CommandExplanationItem;

pub struct HandleResult {
    pub command: String,
    pub explanations: Vec<CommandExplanationItem>,
}
