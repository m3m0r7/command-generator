use anyhow::Result;

use crate::request_engine::HandleResult;

pub fn print_generated_result(result: &HandleResult, explanation_mode: bool) -> Result<()> {
    println!("{}", result.command);
    println!();
    if explanation_mode {
        if result.explanations.is_empty() {
            println!("[]");
        } else {
            let rendered = serde_json::to_string_pretty(&result.explanations)?;
            println!("{}", rendered);
        }
        println!();
    }
    Ok(())
}
