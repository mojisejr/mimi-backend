//! Question Filter Response Models
//!
//! Defines response structures for the QuestionFilter agent with JSON serialization
//! and validation for structured responses from the AI filtering system.

use serde::{Deserialize, Serialize};

/// Response from QuestionFilter agent after AI validation
///
/// Contains the validation result and detailed reason in Thai language
/// as per the "แม่หมอมีมี่" persona requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFilterResponse {
    /// Whether the question is valid for tarot reading
    pub is_valid: bool,
    /// Detailed reason in Thai language explaining the validation decision
    pub reason: String,
}

impl QuestionFilterResponse {
    /// Create a valid response with Thai reason
    ///
    /// # Arguments
    ///
    /// * `reason` - Thai explanation of why the question is valid
    ///
    /// # Returns
    ///
    /// A new QuestionFilterResponse with is_valid=true
    pub fn valid(reason: String) -> Self {
        Self { is_valid: true, reason }
    }

    /// Create an invalid response with Thai reason
    ///
    /// # Arguments
    ///
    /// * `reason` - Thai explanation of why the question is invalid
    ///
    /// # Returns
    ///
    /// A new QuestionFilterResponse with is_valid=false
    pub fn invalid(reason: String) -> Self {
        Self { is_valid: false, reason }
    }

    /// Validate the response format and content
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Response is valid
    /// * `Err(String)` - Reason why response is invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.reason.is_empty() {
            return Err("Response reason cannot be empty".to_string());
        }

        // Basic Thai character check - should contain at least some Thai characters
        if self.is_valid && !self.contains_thai_chars() {
            return Err("Valid response should contain Thai characters".to_string());
        }

        Ok(())
    }

    /// Check if the reason contains Thai characters
    ///
    /// This is a basic validation to ensure Thai persona is maintained
    ///
    /// # Returns
    ///
    /// true if Thai characters are found, false otherwise
    pub fn contains_thai_chars(&self) -> bool {
        self.reason.chars().any(|c| {
            let code_point = c as u32;
            // Basic Thai Unicode range check
            (0x0E00..=0x0E7F).contains(&code_point)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_filter_response_valid_creation() {
        let response = QuestionFilterResponse::valid(
            "คำถามนี้เหมาะสมสำหรับการทำนายดวงชะตา".to_string(),
        );

        assert!(response.is_valid);
        assert_eq!(
            response.reason,
            "คำถามนี้เหมาะสมสำหรับการทำนายดวงชะตา"
        );
    }

    #[test]
    fn test_question_filter_response_invalid_creation() {
        let response = QuestionFilterResponse::invalid(
            "คำถามนี้ไม่เหมาะสมเนื่องจากมีเนื้อหาที่เป็นอันตราย".to_string(),
        );

        assert!(!response.is_valid);
        assert_eq!(
            response.reason,
            "คำถามนี้ไม่เหมาะสมเนื่องจากมีเนื้อหาที่เป็นอันตราย"
        );
    }

    #[test]
    fn test_response_validation_empty_reason() {
        let response = QuestionFilterResponse::valid("".to_string());
        let result = response.validate();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Response reason cannot be empty");
    }

    #[test]
    fn test_response_validation_valid_with_no_thai() {
        let response = QuestionFilterResponse::valid("This is a valid question".to_string());
        let result = response.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("should contain Thai characters"));
    }

    #[test]
    fn test_response_validation_invalid_with_no_thai() {
        let response = QuestionFilterResponse::invalid("This question is inappropriate".to_string());
        let result = response.validate();

        // Invalid responses don't require Thai characters (they could be system messages)
        assert!(result.is_ok());
    }

    #[test]
    fn test_response_validation_valid_with_thai() {
        let response = QuestionFilterResponse::valid(
            "คำถามของคุณเหมาะสมสำหรับการทำนายดวงชะตาค่ะ".to_string(),
        );
        let result = response.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_thai_chars_detection() {
        let thai_response = QuestionFilterResponse::valid("สวัสดีค่ะ".to_string());
        assert!(thai_response.contains_thai_chars());

        let english_response = QuestionFilterResponse::valid("Hello".to_string());
        assert!(!english_response.contains_thai_chars());

        let mixed_response = QuestionFilterResponse::valid("สวัสดี Hello".to_string());
        assert!(mixed_response.contains_thai_chars());
    }

    #[test]
    fn test_json_serialization_deserialization() {
        let original = QuestionFilterResponse::valid(
            "คำถามนี้เหมาะสมสำหรับการทำนาย".to_string(),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&original).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: QuestionFilterResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original.is_valid, deserialized.is_valid);
        assert_eq!(original.reason, deserialized.reason);
    }
}