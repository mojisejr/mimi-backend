//! Soft Delete Tests
//!
//! Tests for the soft delete functionality as per Task #40.
//! Following TDD approach - write tests FIRST (Red Phase).

use chrono::{DateTime, Utc};
use mimivibe_backend::repository::soft_delete::{
    SoftDeletable, SoftDeleteFilter, SoftDeleteOperation,
};
use uuid::Uuid;

// ============================================================================
// Test Fixtures - Mock entities for testing soft delete behavior
// ============================================================================

/// Mock User entity for testing
#[derive(Debug, Clone)]
struct MockUser {
    id: Uuid,
    email: String,
    deleted_at: Option<DateTime<Utc>>,
}

impl SoftDeletable for MockUser {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn set_deleted_at(&mut self, timestamp: Option<DateTime<Utc>>) {
        self.deleted_at = timestamp;
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Mock Job entity for testing
#[derive(Debug, Clone)]
struct MockJob {
    id: Uuid,
    user_id: Uuid,
    #[allow(dead_code)]
    status: String,
    deleted_at: Option<DateTime<Utc>>,
}

impl SoftDeletable for MockJob {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn set_deleted_at(&mut self, timestamp: Option<DateTime<Utc>>) {
        self.deleted_at = timestamp;
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// Mock TarotReading entity for testing
#[derive(Debug, Clone)]
struct MockTarotReading {
    id: Uuid,
    user_id: Uuid,
    #[allow(dead_code)]
    question: String,
    deleted_at: Option<DateTime<Utc>>,
}

impl SoftDeletable for MockTarotReading {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn set_deleted_at(&mut self, timestamp: Option<DateTime<Utc>>) {
        self.deleted_at = timestamp;
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

// ============================================================================
// Unit Tests - Soft Delete Trait Behavior
// ============================================================================

#[test]
fn test_soft_deletable_trait_initial_state() {
    let user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    assert!(user.deleted_at().is_none());
    assert!(!user.is_deleted());
}

#[test]
fn test_soft_deletable_trait_mark_deleted() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let now = Utc::now();
    user.set_deleted_at(Some(now));

    assert!(user.deleted_at().is_some());
    assert!(user.is_deleted());
}

#[test]
fn test_soft_deletable_trait_restore() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: Some(Utc::now()),
    };

    assert!(user.is_deleted());

    user.set_deleted_at(None);

    assert!(!user.is_deleted());
}

// ============================================================================
// Unit Tests - Soft Delete Filter
// ============================================================================

#[test]
fn test_soft_delete_filter_excludes_deleted_items() {
    let users = [
        MockUser {
            id: Uuid::new_v4(),
            email: "active@example.com".to_string(),
            deleted_at: None,
        },
        MockUser {
            id: Uuid::new_v4(),
            email: "deleted@example.com".to_string(),
            deleted_at: Some(Utc::now()),
        },
        MockUser {
            id: Uuid::new_v4(),
            email: "another_active@example.com".to_string(),
            deleted_at: None,
        },
    ];

    let active_users: Vec<_> = users.iter().filter(|u| !u.is_deleted()).collect();

    assert_eq!(active_users.len(), 2);
    assert!(active_users.iter().all(|u| u.deleted_at().is_none()));
}

#[test]
fn test_soft_delete_filter_includes_all_when_include_deleted() {
    let users = [
        MockUser {
            id: Uuid::new_v4(),
            email: "active@example.com".to_string(),
            deleted_at: None,
        },
        MockUser {
            id: Uuid::new_v4(),
            email: "deleted@example.com".to_string(),
            deleted_at: Some(Utc::now()),
        },
    ];

    // When explicitly including deleted items
    let all_users: Vec<_> = users.iter().collect();

    assert_eq!(all_users.len(), 2);
}

// ============================================================================
// Unit Tests - Soft Delete Operation
// ============================================================================

#[test]
fn test_soft_delete_operation_marks_entity_as_deleted() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let operation = SoftDeleteOperation::new();
    let deleted_user = operation.soft_delete(&mut user);

    assert!(deleted_user.is_ok());
    assert!(user.is_deleted());
    assert!(user.deleted_at().is_some());
}

#[test]
fn test_soft_delete_operation_idempotent_when_already_deleted() {
    // Calling soft_delete twice should not error
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: Some(Utc::now()),
    };

    let operation = SoftDeleteOperation::new();
    let result = operation.soft_delete(&mut user);

    // Should succeed (idempotent)
    assert!(result.is_ok());
    assert!(user.is_deleted());
}

#[test]
fn test_soft_delete_preserves_data() {
    let original_email = "test@example.com".to_string();
    let original_id = Uuid::new_v4();

    let mut user = MockUser {
        id: original_id,
        email: original_email.clone(),
        deleted_at: None,
    };

    let operation = SoftDeleteOperation::new();
    let _ = operation.soft_delete(&mut user);

    // Data should be preserved
    assert_eq!(user.id, original_id);
    assert_eq!(user.email, original_email);
    assert!(user.is_deleted());
}

// ============================================================================
// Unit Tests - Restore Operation
// ============================================================================

#[test]
fn test_restore_operation_clears_deleted_at() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: Some(Utc::now()),
    };

    let operation = SoftDeleteOperation::new();
    let result = operation.restore(&mut user);

    assert!(result.is_ok());
    assert!(!user.is_deleted());
    assert!(user.deleted_at().is_none());
}

#[test]
fn test_restore_operation_idempotent_when_not_deleted() {
    // Calling restore on non-deleted item should not error
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let operation = SoftDeleteOperation::new();
    let result = operation.restore(&mut user);

    // Should succeed (idempotent)
    assert!(result.is_ok());
    assert!(!user.is_deleted());
}

// ============================================================================
// Integration Tests - Cascade Soft Delete
// ============================================================================

#[test]
fn test_soft_delete_user_cascades_to_jobs() {
    let user_id = Uuid::new_v4();

    let mut user = MockUser {
        id: user_id,
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let mut jobs = vec![
        MockJob {
            id: Uuid::new_v4(),
            user_id,
            status: "queued".to_string(),
            deleted_at: None,
        },
        MockJob {
            id: Uuid::new_v4(),
            user_id,
            status: "processing".to_string(),
            deleted_at: None,
        },
    ];

    // Simulate cascade delete: parent first, then children
    let operation = SoftDeleteOperation::new();

    // Delete parent (user)
    let _ = operation.soft_delete(&mut user);
    assert!(user.is_deleted());

    // Delete all related jobs (cascade)
    for job in &mut jobs {
        if job.user_id == user_id {
            let _ = operation.soft_delete(job);
        }
    }

    // All jobs should be marked as deleted
    assert!(jobs.iter().all(|j| j.is_deleted()));
}

#[test]
fn test_soft_delete_user_cascades_to_readings() {
    let user_id = Uuid::new_v4();

    let mut user = MockUser {
        id: user_id,
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let mut readings = vec![
        MockTarotReading {
            id: Uuid::new_v4(),
            user_id,
            question: "What is my future?".to_string(),
            deleted_at: None,
        },
        MockTarotReading {
            id: Uuid::new_v4(),
            user_id,
            question: "Will I find love?".to_string(),
            deleted_at: None,
        },
    ];

    let operation = SoftDeleteOperation::new();

    // Delete parent (user)
    let _ = operation.soft_delete(&mut user);
    assert!(user.is_deleted());

    // Delete all related readings (cascade)
    for reading in &mut readings {
        if reading.user_id == user_id {
            let _ = operation.soft_delete(reading);
        }
    }

    // All readings should be marked as deleted
    assert!(readings.iter().all(|r| r.is_deleted()));
}

#[test]
fn test_restore_user_restores_cascade_reverse_order() {
    let user_id = Uuid::new_v4();

    // All entities are currently soft-deleted
    let mut user = MockUser {
        id: user_id,
        email: "test@example.com".to_string(),
        deleted_at: Some(Utc::now()),
    };

    let mut jobs = vec![MockJob {
        id: Uuid::new_v4(),
        user_id,
        status: "queued".to_string(),
        deleted_at: Some(Utc::now()),
    }];

    let mut readings = vec![MockTarotReading {
        id: Uuid::new_v4(),
        user_id,
        question: "Test question".to_string(),
        deleted_at: Some(Utc::now()),
    }];

    let operation = SoftDeleteOperation::new();

    // Restore in REVERSE order: children first, then parent
    // Step 1: Restore readings (children)
    for reading in &mut readings {
        let _ = operation.restore(reading);
    }
    assert!(readings.iter().all(|r| !r.is_deleted()));

    // Step 2: Restore jobs (children)
    for job in &mut jobs {
        let _ = operation.restore(job);
    }
    assert!(jobs.iter().all(|j| !j.is_deleted()));

    // Step 3: Restore user (parent) - last
    let _ = operation.restore(&mut user);
    assert!(!user.is_deleted());
}

// ============================================================================
// Unit Tests - SQL Filter Helper
// ============================================================================

#[test]
fn test_soft_delete_filter_generates_correct_sql() {
    let filter = SoftDeleteFilter::new();

    // Default: exclude deleted
    assert_eq!(filter.where_clause(), "deleted_at IS NULL");

    // Include deleted
    let filter_with_deleted = SoftDeleteFilter::include_deleted();
    assert_eq!(filter_with_deleted.where_clause(), "1=1");

    // Only deleted
    let filter_only_deleted = SoftDeleteFilter::only_deleted();
    assert_eq!(filter_only_deleted.where_clause(), "deleted_at IS NOT NULL");
}

#[test]
fn test_soft_delete_filter_with_alias() {
    let filter = SoftDeleteFilter::new().with_alias("u");

    assert_eq!(filter.where_clause(), "u.deleted_at IS NULL");
}

#[test]
fn test_soft_delete_filter_combined_conditions() {
    let filter = SoftDeleteFilter::new();
    let base_condition = "status = 'active'";

    let combined = format!("{} AND {}", base_condition, filter.where_clause());

    assert_eq!(combined, "status = 'active' AND deleted_at IS NULL");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_soft_delete_multiple_times_is_idempotent() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let operation = SoftDeleteOperation::new();

    // Delete first time
    let result1 = operation.soft_delete(&mut user);
    let deleted_at_1 = user.deleted_at();

    // Delete second time (should be idempotent)
    let result2 = operation.soft_delete(&mut user);
    let deleted_at_2 = user.deleted_at();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Timestamp should remain the same (idempotent)
    assert_eq!(deleted_at_1, deleted_at_2);
}

#[test]
fn test_restore_after_soft_delete_cycle() {
    let mut user = MockUser {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        deleted_at: None,
    };

    let operation = SoftDeleteOperation::new();

    // Cycle: delete -> restore -> delete -> restore
    assert!(!user.is_deleted());

    let _ = operation.soft_delete(&mut user);
    assert!(user.is_deleted());

    let _ = operation.restore(&mut user);
    assert!(!user.is_deleted());

    let _ = operation.soft_delete(&mut user);
    assert!(user.is_deleted());

    let _ = operation.restore(&mut user);
    assert!(!user.is_deleted());
}

#[test]
fn test_filter_empty_collection() {
    let users: Vec<MockUser> = vec![];

    let active_users: Vec<_> = users.iter().filter(|u| !u.is_deleted()).collect();

    assert!(active_users.is_empty());
}

#[test]
fn test_filter_all_deleted_collection() {
    let users = [
        MockUser {
            id: Uuid::new_v4(),
            email: "deleted1@example.com".to_string(),
            deleted_at: Some(Utc::now()),
        },
        MockUser {
            id: Uuid::new_v4(),
            email: "deleted2@example.com".to_string(),
            deleted_at: Some(Utc::now()),
        },
    ];

    let active_users: Vec<_> = users.iter().filter(|u| !u.is_deleted()).collect();

    assert!(active_users.is_empty());
}
