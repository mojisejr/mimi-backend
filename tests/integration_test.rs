//! Integration tests for MimiVibe backend
//!
//! Tests the agent pipeline and API endpoints.

#[cfg(test)]
mod tests {
    use mimivibe_backend::agents::question_filter;

    #[test]
    fn test_question_filter_validates_empty_question() {
        let result = question_filter::filter_question("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_question_filter_accepts_valid_question() {
        let result = question_filter::filter_question("ความรักของฉันจะเป็นอย่างไร");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
