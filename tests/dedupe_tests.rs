//! Dedupe Mechanism Tests
//!
//! Tests for idempotent job creation using dedupe_key and UPSERT pattern.
//! These tests verify the question normalization, dedupe key generation,
//! and idempotent job creation functionality.

use mimivibe_backend::repository::dedupe::{
    generate_dedupe_key, normalize_question, DedupeError, IdempotentJobResult,
};
use uuid::Uuid;

/// Test: dedupe_key_generated_correctly
/// Verify hash generation produces consistent results for same question
#[test]
fn test_dedupe_key_generated_correctly() {
    let user_id = Uuid::new_v4();
    let question = "อยากรู้เรื่องการงานในปีหน้า";

    let key1 = generate_dedupe_key(&user_id, question);
    let key2 = generate_dedupe_key(&user_id, question);

    // Same user + same question should produce same dedupe key
    assert_eq!(key1, key2, "Same input should generate same dedupe key");

    // Key should be a valid SHA256 hex string (64 characters)
    assert_eq!(
        key1.len(),
        64,
        "Dedupe key should be 64 hex characters (SHA256)"
    );

    // Key should only contain hex characters
    assert!(
        key1.chars().all(|c| c.is_ascii_hexdigit()),
        "Dedupe key should only contain hex characters"
    );
}

/// Test: different_users_different_keys
/// Different users asking same question should get different dedupe keys
#[test]
fn test_different_users_different_keys() {
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let question = "อยากรู้เรื่องความรัก";

    let key1 = generate_dedupe_key(&user1, question);
    let key2 = generate_dedupe_key(&user2, question);

    assert_ne!(
        key1, key2,
        "Different users should have different dedupe keys for same question"
    );
}

/// Test: different_questions_different_keys
/// Same user asking different questions should get different dedupe keys
#[test]
fn test_different_questions_different_keys() {
    let user_id = Uuid::new_v4();
    let question1 = "อยากรู้เรื่องการงาน";
    let question2 = "อยากรู้เรื่องความรัก";

    let key1 = generate_dedupe_key(&user_id, question1);
    let key2 = generate_dedupe_key(&user_id, question2);

    assert_ne!(
        key1, key2,
        "Different questions should have different dedupe keys"
    );
}

/// Test: question_normalization - basic trimming
/// Verify questions with leading/trailing whitespace are normalized
#[test]
fn test_question_normalization_trim() {
    let q1 = normalize_question("  hello  ");
    let q2 = normalize_question("hello");

    assert_eq!(q1, q2, "Leading/trailing whitespace should be trimmed");
    assert_eq!(q1, "hello");
}

/// Test: question_normalization - lowercase conversion
/// Verify questions are converted to lowercase
#[test]
fn test_question_normalization_lowercase() {
    let q1 = normalize_question("Hello World");
    let q2 = normalize_question("hello world");

    assert_eq!(q1, q2, "Questions should be normalized to lowercase");
    assert_eq!(q1, "hello world");
}

/// Test: question_normalization - whitespace collapse
/// Verify multiple spaces are collapsed into single space
#[test]
fn test_question_normalization_whitespace_collapse() {
    let q1 = normalize_question("hello    world");
    let q2 = normalize_question("hello world");

    assert_eq!(
        q1, q2,
        "Multiple spaces should be collapsed to single space"
    );
    assert_eq!(q1, "hello world");
}

/// Test: question_normalization - all transformations combined
/// Verify variations with same content get same normalized form
#[test]
fn test_question_normalization_combined() {
    // These should all normalize to "hello?"
    let variations_no_space = ["Hello?", "hello?", "  hello?  ", "  HELLO?  "];

    let normalized_no_space: Vec<String> = variations_no_space
        .iter()
        .map(|q| normalize_question(q))
        .collect();

    let first_no_space = &normalized_no_space[0];
    for (i, norm) in normalized_no_space.iter().enumerate() {
        assert_eq!(
            norm, first_no_space,
            "Variation '{}' should normalize the same as others",
            variations_no_space[i]
        );
    }
    assert_eq!(first_no_space, "hello?");

    // These should all normalize to "hello ?" (with space before ?)
    let variations_with_space = ["hello ?", "hello  ?", "  hello   ?  ", "HELLO ?"];

    let normalized_with_space: Vec<String> = variations_with_space
        .iter()
        .map(|q| normalize_question(q))
        .collect();

    let first_with_space = &normalized_with_space[0];
    for (i, norm) in normalized_with_space.iter().enumerate() {
        assert_eq!(
            norm, first_with_space,
            "Variation '{}' should normalize the same as others",
            variations_with_space[i]
        );
    }
    assert_eq!(first_with_space, "hello ?");
}

/// Test: question_normalization - Thai characters preserved
/// Verify Thai characters are not affected by normalization (Thai doesn't have case)
#[test]
fn test_question_normalization_thai() {
    let q1 = normalize_question("  อยากรู้เรื่องความรัก  ");
    let q2 = normalize_question("อยากรู้เรื่องความรัก");

    assert_eq!(q1, q2, "Thai questions should be normalized correctly");
    assert_eq!(q1, "อยากรู้เรื่องความรัก");
}

/// Test: question_normalization - mixed Thai and English
#[test]
fn test_question_normalization_mixed_language() {
    let q1 = normalize_question("  Future ความรัก  ");
    let q2 = normalize_question("future ความรัก");

    assert_eq!(
        q1, q2,
        "Mixed language questions should be normalized correctly"
    );
    assert_eq!(q1, "future ความรัก");
}

/// Test: normalized questions produce same dedupe key
/// Core idempotency test - variations of same question should produce same key
#[test]
fn test_normalized_questions_same_dedupe_key() {
    let user_id = Uuid::new_v4();

    let variations = [
        "What is my future?",
        "what is my future?",
        "  what is my future?  ",
        "WHAT IS MY FUTURE?",
        "what  is   my   future?",
    ];

    let keys: Vec<String> = variations
        .iter()
        .map(|q| generate_dedupe_key(&user_id, q))
        .collect();

    // All variations should produce the same dedupe key
    let first = &keys[0];
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(
            key, first,
            "Question '{}' should produce same dedupe key as '{}'",
            variations[i], variations[0]
        );
    }
}

/// Test: empty question handling
#[test]
fn test_empty_question_normalization() {
    let q = normalize_question("");
    assert_eq!(q, "", "Empty question should remain empty");

    let q_spaces = normalize_question("   ");
    assert_eq!(q_spaces, "", "Whitespace-only question should become empty");
}

/// Test: IdempotentJobResult variants
#[test]
fn test_idempotent_job_result_created() {
    let job_id = Uuid::new_v4();
    let result = IdempotentJobResult::Created { job_id };

    match result {
        IdempotentJobResult::Created { job_id: id } => {
            assert_eq!(id, job_id);
        }
        _ => panic!("Expected Created variant"),
    }
}

#[test]
fn test_idempotent_job_result_existing() {
    let job_id = Uuid::new_v4();
    let result = IdempotentJobResult::Existing { job_id };

    match result {
        IdempotentJobResult::Existing { job_id: id } => {
            assert_eq!(id, job_id);
        }
        _ => panic!("Expected Existing variant"),
    }
}

/// Test: DedupeError variants
#[test]
fn test_dedupe_error_display() {
    let err = DedupeError::EmptyQuestion;
    assert!(err.to_string().contains("empty"));

    let err = DedupeError::InvalidUserId("bad-uuid".to_string());
    assert!(err.to_string().contains("Invalid user ID"));

    let err = DedupeError::DatabaseError("connection failed".to_string());
    assert!(err.to_string().contains("Database error"));
}
