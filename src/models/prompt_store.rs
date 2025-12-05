//! Stored Prompt Model
//!
//! Represents a prompt record as stored in the database.
//! Used for managing AI agent prompts with version control and soft-delete capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Valid agent names for prompts
pub const VALID_AGENT_NAMES: [&str; 3] = ["question_filter", "question_analyzer", "reading_agent"];

/// Prompt record as stored in database
///
/// Maps directly to the `prompts` table schema:
/// - id: UUID primary key
/// - agent_name: One of question_filter, question_analyzer, reading_agent
/// - prompt_content: Full prompt template with {placeholders}
/// - version: Integer version number (starts at 1)
/// - is_active: Soft-delete flag (true = active)
/// - created_at: Creation timestamp
/// - updated_at: Last update timestamp
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StoredPrompt {
    /// Primary key: UUID auto-generated
    pub id: Uuid,
    /// Agent identifier: question_filter, question_analyzer, reading_agent
    pub agent_name: String,
    /// Full prompt content with {placeholders} for template rendering
    pub prompt_content: String,
    /// Version number (starts at 1, incremented on updates)
    pub version: i32,
    /// Soft-delete flag: true = active, false = deleted/inactive
    pub is_active: bool,
    /// Timestamp when prompt was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when prompt was last updated
    pub updated_at: DateTime<Utc>,
}

impl StoredPrompt {
    /// Verify this is a valid agent prompt
    ///
    /// A prompt is considered valid if:
    /// - agent_name is one of the allowed values
    /// - is_active is true
    /// - version is greater than 0
    ///
    /// # Returns
    ///
    /// `true` if the prompt is valid, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use mimivibe_backend::models::StoredPrompt;
    /// use chrono::Utc;
    /// use uuid::Uuid;
    ///
    /// let prompt = StoredPrompt {
    ///     id: Uuid::new_v4(),
    ///     agent_name: "question_filter".to_string(),
    ///     prompt_content: "Test prompt".to_string(),
    ///     version: 1,
    ///     is_active: true,
    ///     created_at: Utc::now(),
    ///     updated_at: Utc::now(),
    /// };
    ///
    /// assert!(prompt.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        matches!(
            self.agent_name.as_str(),
            "question_filter" | "question_analyzer" | "reading_agent"
        ) && self.is_active
            && self.version > 0
    }

    /// Check if the agent name is valid
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The agent name to validate
    ///
    /// # Returns
    ///
    /// `true` if the agent name is valid, `false` otherwise
    pub fn is_valid_agent_name(agent_name: &str) -> bool {
        VALID_AGENT_NAMES.contains(&agent_name)
    }

    /// Get the prompt content for template rendering
    ///
    /// Returns a reference to the prompt_content field
    pub fn content(&self) -> &str {
        &self.prompt_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_prompt(agent_name: &str, version: i32, is_active: bool) -> StoredPrompt {
        StoredPrompt {
            id: Uuid::new_v4(),
            agent_name: agent_name.to_string(),
            prompt_content: "Test prompt content with {question}".to_string(),
            version,
            is_active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_stored_prompt_is_valid_with_question_filter() {
        let prompt = create_test_prompt("question_filter", 1, true);
        assert!(prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_is_valid_with_question_analyzer() {
        let prompt = create_test_prompt("question_analyzer", 2, true);
        assert!(prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_is_valid_with_reading_agent() {
        let prompt = create_test_prompt("reading_agent", 1, true);
        assert!(prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_invalid_agent_name() {
        let prompt = create_test_prompt("invalid_agent", 1, true);
        assert!(!prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_invalid_when_inactive() {
        let prompt = create_test_prompt("question_filter", 1, false);
        assert!(!prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_invalid_when_version_zero() {
        let prompt = create_test_prompt("question_filter", 0, true);
        assert!(!prompt.is_valid());
    }

    #[test]
    fn test_stored_prompt_invalid_when_version_negative() {
        let prompt = create_test_prompt("question_filter", -1, true);
        assert!(!prompt.is_valid());
    }

    #[test]
    fn test_is_valid_agent_name() {
        assert!(StoredPrompt::is_valid_agent_name("question_filter"));
        assert!(StoredPrompt::is_valid_agent_name("question_analyzer"));
        assert!(StoredPrompt::is_valid_agent_name("reading_agent"));
        assert!(!StoredPrompt::is_valid_agent_name("invalid_agent"));
        assert!(!StoredPrompt::is_valid_agent_name(""));
    }

    #[test]
    fn test_content_method() {
        let prompt = create_test_prompt("question_filter", 1, true);
        assert_eq!(prompt.content(), "Test prompt content with {question}");
    }
}
