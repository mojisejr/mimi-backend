//! Dedupe Mechanism Module
//!
//! Implements idempotent job creation using dedupe_key and UPSERT pattern.
//! This module prevents duplicate tarot readings by generating unique keys
//! from user_id and normalized questions.
//!
//! # Design Principles
//! - Generate dedupe_key from: sha256(user_id || '|' || normalized_question)
//! - Question normalization: lowercase, trim, collapse whitespace
//! - Use SQL INSERT ... ON CONFLICT DO NOTHING pattern for idempotency
//! - Database is source-of-truth, Redis is ephemeral queue only
//!
//! # Usage
//! ```ignore
//! use mimivibe_backend::repository::dedupe::{generate_dedupe_key, normalize_question};
//! use uuid::Uuid;
//!
//! let user_id = Uuid::new_v4();
//! let question = "  What is my future?  ";
//!
//! // Normalize question first
//! let normalized = normalize_question(question);
//!
//! // Generate dedupe key
//! let dedupe_key = generate_dedupe_key(&user_id, question);
//!
//! // Use dedupe_key for idempotent job creation
//! ```

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Result of idempotent job creation.
///
/// When creating a job with a dedupe_key, one of two outcomes is possible:
/// - `Created`: A new job was created (first request with this dedupe_key)
/// - `Existing`: Job already exists (duplicate request detected and prevented)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotentJobResult {
    /// A new job was successfully created
    Created {
        /// The newly created job's ID
        job_id: Uuid,
    },
    /// An existing job was found (duplicate prevented)
    Existing {
        /// The existing job's ID
        job_id: Uuid,
    },
}

impl IdempotentJobResult {
    /// Returns the job_id regardless of whether it was created or existing
    pub fn job_id(&self) -> Uuid {
        match self {
            IdempotentJobResult::Created { job_id } => *job_id,
            IdempotentJobResult::Existing { job_id } => *job_id,
        }
    }

    /// Returns true if a new job was created
    pub fn is_created(&self) -> bool {
        matches!(self, IdempotentJobResult::Created { .. })
    }

    /// Returns true if an existing job was found (duplicate)
    pub fn is_existing(&self) -> bool {
        matches!(self, IdempotentJobResult::Existing { .. })
    }
}

/// Error types for dedupe operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedupeError {
    /// Question is empty or contains only whitespace
    EmptyQuestion,
    /// User ID is invalid
    InvalidUserId(String),
    /// Database operation failed
    DatabaseError(String),
    /// Transaction failed
    TransactionError(String),
    /// Job not found when querying by dedupe_key
    JobNotFound(String),
}

impl std::fmt::Display for DedupeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DedupeError::EmptyQuestion => {
                write!(f, "Question is empty or contains only whitespace")
            }
            DedupeError::InvalidUserId(id) => write!(f, "Invalid user ID: {}", id),
            DedupeError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            DedupeError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            DedupeError::JobNotFound(key) => write!(f, "Job not found for dedupe_key: {}", key),
        }
    }
}

impl std::error::Error for DedupeError {}

/// Normalizes a question for consistent dedupe key generation.
///
/// Applies the following transformations:
/// 1. Trim leading and trailing whitespace
/// 2. Convert to lowercase (for ASCII characters)
/// 3. Collapse multiple whitespace characters into single space
///
/// # Arguments
/// * `question` - The original question string
///
/// # Returns
/// Normalized question string
///
/// # Example
/// ```
/// use mimivibe_backend::repository::dedupe::normalize_question;
///
/// assert_eq!(normalize_question("  Hello   World  "), "hello world");
/// assert_eq!(normalize_question("WHAT?"), "what?");
/// assert_eq!(normalize_question("อยากรู้เรื่องความรัก"), "อยากรู้เรื่องความรัก");
/// ```
pub fn normalize_question(question: &str) -> String {
    // Step 1: Trim leading and trailing whitespace
    let trimmed = question.trim();

    // Step 2: Convert to lowercase
    let lowercase = trimmed.to_lowercase();

    // Step 3: Collapse multiple whitespace into single space
    let mut result = String::with_capacity(lowercase.len());
    let mut prev_was_whitespace = false;

    for c in lowercase.chars() {
        if c.is_whitespace() {
            if !prev_was_whitespace {
                result.push(' ');
            }
            prev_was_whitespace = true;
        } else {
            result.push(c);
            prev_was_whitespace = false;
        }
    }

    // Trim any trailing space that might have been added
    result.trim().to_string()
}

/// Generates a dedupe key from user_id and question.
///
/// The dedupe key is generated using SHA256:
/// `sha256(user_id || '|' || normalized_question)`
///
/// This ensures that:
/// - Same user + same question (after normalization) = same dedupe_key
/// - Different users + same question = different dedupe_key
/// - Same user + different questions = different dedupe_key
///
/// # Arguments
/// * `user_id` - The user's UUID
/// * `question` - The question string (will be normalized internally)
///
/// # Returns
/// A 64-character hexadecimal string representing the SHA256 hash
///
/// # Example
/// ```
/// use mimivibe_backend::repository::dedupe::generate_dedupe_key;
/// use uuid::Uuid;
///
/// let user_id = Uuid::new_v4();
/// let question = "What is my future?";
///
/// let key = generate_dedupe_key(&user_id, question);
/// assert_eq!(key.len(), 64); // SHA256 produces 64 hex characters
/// ```
pub fn generate_dedupe_key(user_id: &Uuid, question: &str) -> String {
    // Normalize the question first
    let normalized = normalize_question(question);

    // Create the input string: user_id|normalized_question
    let input = format!("{}|{}", user_id, normalized);

    // Generate SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    // Convert to hexadecimal string
    hex::encode(result)
}

/// Validates a dedupe key format.
///
/// A valid dedupe key is a 64-character hexadecimal string (SHA256).
///
/// # Arguments
/// * `dedupe_key` - The dedupe key to validate
///
/// # Returns
/// `true` if the dedupe key is valid, `false` otherwise
///
/// # Example
/// ```
/// use mimivibe_backend::repository::dedupe::is_valid_dedupe_key;
///
/// assert!(is_valid_dedupe_key("a".repeat(64).as_str()));
/// assert!(!is_valid_dedupe_key("too_short"));
/// assert!(!is_valid_dedupe_key("invalid-chars-here!!"));
/// ```
pub fn is_valid_dedupe_key(dedupe_key: &str) -> bool {
    dedupe_key.len() == 64 && dedupe_key.chars().all(|c| c.is_ascii_hexdigit())
}

/// SQL pattern for idempotent job insertion.
///
/// This constant provides the SQL template for UPSERT operations:
/// ```sql
/// INSERT INTO jobs (id, ..., dedupe_key, ...)
/// VALUES ($1, ..., $N, ...)
/// ON CONFLICT (dedupe_key) DO NOTHING
/// RETURNING id;
/// ```
///
/// If the insert returns nothing (due to conflict), query for existing job:
/// ```sql
/// SELECT id FROM jobs WHERE dedupe_key = $1;
/// ```
pub const UPSERT_JOB_SQL: &str = r#"
INSERT INTO jobs (
    id, job_type, schema_version, prompt_version, dedupe_key,
    payload, status, attempts, max_attempts, visibility_timeout_secs,
    created_at, updated_at
)
VALUES (
    $1, $2, $3, $4, $5,
    $6, 'queued', 0, $7, $8,
    NOW(), NOW()
)
ON CONFLICT (dedupe_key) DO NOTHING
RETURNING id
"#;

/// SQL to query existing job by dedupe_key.
pub const QUERY_BY_DEDUPE_KEY_SQL: &str = r#"
SELECT id FROM jobs WHERE dedupe_key = $1
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_question_basic() {
        assert_eq!(normalize_question("hello"), "hello");
        assert_eq!(normalize_question("HELLO"), "hello");
        assert_eq!(normalize_question("  hello  "), "hello");
    }

    #[test]
    fn test_normalize_question_whitespace() {
        assert_eq!(normalize_question("hello   world"), "hello world");
        assert_eq!(normalize_question("  hello   world  "), "hello world");
        assert_eq!(normalize_question("\t\nhello\t\nworld\t\n"), "hello world");
    }

    #[test]
    fn test_generate_dedupe_key_consistency() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let question = "test question";

        let key1 = generate_dedupe_key(&user_id, question);
        let key2 = generate_dedupe_key(&user_id, question);

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64);
    }

    #[test]
    fn test_generate_dedupe_key_different_users() {
        let user1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let user2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let question = "test question";

        let key1 = generate_dedupe_key(&user1, question);
        let key2 = generate_dedupe_key(&user2, question);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_idempotent_job_result() {
        let job_id = Uuid::new_v4();

        let created = IdempotentJobResult::Created { job_id };
        assert!(created.is_created());
        assert!(!created.is_existing());
        assert_eq!(created.job_id(), job_id);

        let existing = IdempotentJobResult::Existing { job_id };
        assert!(!existing.is_created());
        assert!(existing.is_existing());
        assert_eq!(existing.job_id(), job_id);
    }

    #[test]
    fn test_is_valid_dedupe_key() {
        // Valid key (64 hex characters)
        let valid_key = "a".repeat(64);
        assert!(is_valid_dedupe_key(&valid_key));

        // Invalid: too short
        assert!(!is_valid_dedupe_key("abc123"));

        // Invalid: contains non-hex characters
        assert!(!is_valid_dedupe_key(&"g".repeat(64)));

        // Invalid: too long
        assert!(!is_valid_dedupe_key(&"a".repeat(65)));
    }
}
