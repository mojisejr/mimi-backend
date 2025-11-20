//! Question Filter Agent
//!
//! Validates and filters user questions before processing through the tarot pipeline.
//! Part of the LangGraph-style agent workflow.

/// Filter a user question for validity
pub fn filter_question(question: &str) -> Result<bool, String> {
    // TODO: Implement question validation logic
    // - Check for minimum length
    // - Validate Thai language content
    // - Filter inappropriate questions
    Ok(!question.is_empty())
}
