use anyhow::{Result, anyhow};

use crate::prompter::{ClarificationKind, ClarificationPrompter};
use crate::request_engine::guards::normalize_question_text;
use crate::request_engine::state::RuntimeState;

pub fn handle_question(
    kind: ClarificationKind,
    question: String,
    prompter: &mut Option<&mut dyn ClarificationPrompter>,
    state: &mut RuntimeState,
) -> Result<()> {
    state.ensure_question_capacity()?;
    let normalized = normalize_question_text(&question);
    state.register_question(normalized, &question)?;

    let answer = match prompter.as_deref_mut() {
        Some(asker) => asker.ask(kind, &question)?,
        None => {
            let mode = match kind {
                ClarificationKind::YesNo => "y/n",
                ClarificationKind::Text => "text",
            };
            return Err(anyhow!(
                "model requested clarification ('{}') but --once mode cannot answer {}; run interactive mode",
                question,
                mode
            ));
        }
    };

    state.push_clarification(question, answer);
    state.clear_feedback();
    Ok(())
}
