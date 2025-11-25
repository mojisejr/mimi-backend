//! Soft Delete Module
//!
//! Provides soft delete functionality for database entities.
//! All tables use `deleted_at TIMESTAMPTZ` column for soft deletion.
//!
//! # Design Principles
//! - Soft delete preserves data integrity (no physical deletion)
//! - All SELECT queries should use `WHERE deleted_at IS NULL` by default
//! - Cascade soft delete: parent entities delete children
//! - Restore operation: children restored first, then parent
//!
//! # Usage
//! ```ignore
//! use mimivibe_backend::repository::soft_delete::{SoftDeletable, SoftDeleteOperation};
//!
//! // Mark entity as deleted
//! let operation = SoftDeleteOperation::new();
//! operation.soft_delete(&mut entity);
//!
//! // Check if deleted
//! if entity.is_deleted() {
//!     // Handle deleted entity
//! }
//!
//! // Restore entity
//! operation.restore(&mut entity);
//! ```

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Trait for entities that support soft deletion.
///
/// All database entities with `deleted_at` column should implement this trait.
/// This enables consistent soft delete behavior across all tables.
///
/// # Tables with soft delete support
/// - users
/// - jobs
/// - tarot_readings
/// - payments
/// - wallet_transactions
/// - job_attempts
/// - user_payment_methods
/// - user_invites
pub trait SoftDeletable {
    /// Returns the soft delete timestamp if the entity is deleted.
    fn deleted_at(&self) -> Option<DateTime<Utc>>;

    /// Sets the soft delete timestamp.
    /// - `Some(timestamp)` marks entity as deleted
    /// - `None` restores the entity
    fn set_deleted_at(&mut self, timestamp: Option<DateTime<Utc>>);

    /// Returns the entity's unique identifier.
    fn id(&self) -> Uuid;

    /// Returns true if the entity is soft deleted.
    ///
    /// # Example
    /// ```ignore
    /// if entity.is_deleted() {
    ///     // Entity is soft deleted, should not be returned in queries
    /// }
    /// ```
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// Operations for soft delete functionality.
///
/// Provides methods to soft delete and restore entities.
/// All operations are idempotent - calling multiple times has no additional effect.
#[derive(Debug, Clone, Default)]
pub struct SoftDeleteOperation {
    /// Optional timestamp to use for deletion.
    /// If None, uses current UTC time.
    timestamp: Option<DateTime<Utc>>,
}

impl SoftDeleteOperation {
    /// Creates a new soft delete operation with current timestamp.
    pub fn new() -> Self {
        Self { timestamp: None }
    }

    /// Creates a soft delete operation with a specific timestamp.
    /// Useful for testing or batch operations.
    pub fn with_timestamp(timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp: Some(timestamp),
        }
    }

    /// Soft deletes an entity by setting `deleted_at` to current timestamp.
    ///
    /// This operation is idempotent - if the entity is already deleted,
    /// the existing `deleted_at` timestamp is preserved.
    ///
    /// # Arguments
    /// * `entity` - Mutable reference to the entity to delete
    ///
    /// # Returns
    /// * `Ok(())` - Entity was successfully soft deleted
    /// * `Err(SoftDeleteError)` - Operation failed (currently infallible)
    ///
    /// # Example
    /// ```ignore
    /// let operation = SoftDeleteOperation::new();
    /// operation.soft_delete(&mut user)?;
    /// assert!(user.is_deleted());
    /// ```
    pub fn soft_delete<T: SoftDeletable>(&self, entity: &mut T) -> Result<(), SoftDeleteError> {
        // Idempotent: if already deleted, preserve existing timestamp
        if entity.is_deleted() {
            return Ok(());
        }

        let timestamp = self.timestamp.unwrap_or_else(Utc::now);
        entity.set_deleted_at(Some(timestamp));
        Ok(())
    }

    /// Restores a soft deleted entity by clearing `deleted_at`.
    ///
    /// This operation is idempotent - if the entity is not deleted,
    /// no changes are made.
    ///
    /// # Arguments
    /// * `entity` - Mutable reference to the entity to restore
    ///
    /// # Returns
    /// * `Ok(())` - Entity was successfully restored
    /// * `Err(SoftDeleteError)` - Operation failed (currently infallible)
    ///
    /// # Example
    /// ```ignore
    /// let operation = SoftDeleteOperation::new();
    /// operation.restore(&mut user)?;
    /// assert!(!user.is_deleted());
    /// ```
    pub fn restore<T: SoftDeletable>(&self, entity: &mut T) -> Result<(), SoftDeleteError> {
        // Idempotent: if not deleted, no changes needed
        if !entity.is_deleted() {
            return Ok(());
        }

        entity.set_deleted_at(None);
        Ok(())
    }

    /// Soft deletes multiple entities in batch.
    ///
    /// # Arguments
    /// * `entities` - Mutable slice of entities to delete
    ///
    /// # Returns
    /// * `Ok(count)` - Number of entities that were soft deleted
    pub fn soft_delete_batch<T: SoftDeletable>(
        &self,
        entities: &mut [T],
    ) -> Result<usize, SoftDeleteError> {
        let mut count = 0;
        for entity in entities {
            if !entity.is_deleted() {
                self.soft_delete(entity)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Restores multiple entities in batch.
    ///
    /// # Arguments
    /// * `entities` - Mutable slice of entities to restore
    ///
    /// # Returns
    /// * `Ok(count)` - Number of entities that were restored
    pub fn restore_batch<T: SoftDeletable>(
        &self,
        entities: &mut [T],
    ) -> Result<usize, SoftDeleteError> {
        let mut count = 0;
        for entity in entities {
            if entity.is_deleted() {
                self.restore(entity)?;
                count += 1;
            }
        }
        Ok(count)
    }
}

/// SQL filter helper for soft delete queries.
///
/// Generates WHERE clause conditions for filtering soft deleted entities.
/// By default, excludes all deleted entities (deleted_at IS NULL).
///
/// # Example
/// ```
/// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
/// let filter = SoftDeleteFilter::new();
/// let sql = format!("SELECT * FROM users WHERE {}", filter.where_clause());
/// // Result: "SELECT * FROM users WHERE deleted_at IS NULL"
/// ```
#[derive(Debug, Clone)]
pub struct SoftDeleteFilter {
    /// Whether to include deleted entities in results
    include_deleted: bool,
    /// Whether to show only deleted entities
    only_deleted: bool,
    /// Table alias for qualified column names
    alias: Option<String>,
}

impl Default for SoftDeleteFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftDeleteFilter {
    /// Creates a new filter that excludes deleted entities (default behavior).
    ///
    /// # Example
    /// ```
    /// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
    /// let filter = SoftDeleteFilter::new();
    /// assert_eq!(filter.where_clause(), "deleted_at IS NULL");
    /// ```
    pub fn new() -> Self {
        Self {
            include_deleted: false,
            only_deleted: false,
            alias: None,
        }
    }

    /// Creates a filter that includes all entities (both active and deleted).
    ///
    /// # Example
    /// ```
    /// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
    /// let filter = SoftDeleteFilter::include_deleted();
    /// assert_eq!(filter.where_clause(), "1=1"); // No filter
    /// ```
    pub fn include_deleted() -> Self {
        Self {
            include_deleted: true,
            only_deleted: false,
            alias: None,
        }
    }

    /// Creates a filter that shows only deleted entities.
    ///
    /// # Example
    /// ```
    /// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
    /// let filter = SoftDeleteFilter::only_deleted();
    /// assert_eq!(filter.where_clause(), "deleted_at IS NOT NULL");
    /// ```
    pub fn only_deleted() -> Self {
        Self {
            include_deleted: false,
            only_deleted: true,
            alias: None,
        }
    }

    /// Sets a table alias for qualified column names.
    ///
    /// # Arguments
    /// * `alias` - Table alias (e.g., "u" for "users u")
    ///
    /// # Example
    /// ```
    /// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
    /// let filter = SoftDeleteFilter::new().with_alias("u");
    /// assert_eq!(filter.where_clause(), "u.deleted_at IS NULL");
    /// ```
    pub fn with_alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    /// Generates the WHERE clause condition for SQL queries.
    ///
    /// # Returns
    /// SQL condition string for filtering soft deleted entities
    ///
    /// # Examples
    /// - Default: `"deleted_at IS NULL"`
    /// - Include deleted: `"1=1"`
    /// - Only deleted: `"deleted_at IS NOT NULL"`
    /// - With alias: `"u.deleted_at IS NULL"`
    pub fn where_clause(&self) -> String {
        let column = match &self.alias {
            Some(alias) => format!("{}.deleted_at", alias),
            None => "deleted_at".to_string(),
        };

        if self.include_deleted {
            "1=1".to_string()
        } else if self.only_deleted {
            format!("{} IS NOT NULL", column)
        } else {
            format!("{} IS NULL", column)
        }
    }

    /// Generates SQL fragment for combining with existing conditions using AND.
    ///
    /// # Example
    /// ```
    /// use mimivibe_backend::repository::soft_delete::SoftDeleteFilter;
    /// let filter = SoftDeleteFilter::new();
    /// let sql = format!(
    ///     "SELECT * FROM users WHERE status = 'active' AND {}",
    ///     filter.and_clause()
    /// );
    /// ```
    pub fn and_clause(&self) -> String {
        self.where_clause()
    }
}

/// Error types for soft delete operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoftDeleteError {
    /// Entity was not found
    NotFound(Uuid),
    /// Entity is already deleted (for operations that require active entity)
    AlreadyDeleted(Uuid),
    /// Database operation failed
    DatabaseError(String),
    /// Transaction failed
    TransactionError(String),
}

impl std::fmt::Display for SoftDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoftDeleteError::NotFound(id) => write!(f, "Entity with id {} not found", id),
            SoftDeleteError::AlreadyDeleted(id) => {
                write!(f, "Entity with id {} is already deleted", id)
            }
            SoftDeleteError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SoftDeleteError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for SoftDeleteError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test entity for unit tests
    struct TestEntity {
        id: Uuid,
        deleted_at: Option<DateTime<Utc>>,
    }

    impl SoftDeletable for TestEntity {
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

    #[test]
    fn test_soft_delete_operation_new() {
        let op = SoftDeleteOperation::new();
        assert!(op.timestamp.is_none());
    }

    #[test]
    fn test_soft_delete_operation_with_timestamp() {
        let ts = Utc::now();
        let op = SoftDeleteOperation::with_timestamp(ts);
        assert_eq!(op.timestamp, Some(ts));
    }

    #[test]
    fn test_soft_delete_filter_default() {
        let filter = SoftDeleteFilter::default();
        assert_eq!(filter.where_clause(), "deleted_at IS NULL");
    }

    #[test]
    fn test_soft_delete_error_display() {
        let id = Uuid::new_v4();

        let err = SoftDeleteError::NotFound(id);
        assert!(err.to_string().contains("not found"));

        let err = SoftDeleteError::AlreadyDeleted(id);
        assert!(err.to_string().contains("already deleted"));

        let err = SoftDeleteError::DatabaseError("test error".to_string());
        assert!(err.to_string().contains("Database error"));

        let err = SoftDeleteError::TransactionError("tx failed".to_string());
        assert!(err.to_string().contains("Transaction error"));
    }

    #[test]
    fn test_batch_soft_delete() {
        let mut entities = vec![
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: None,
            },
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: None,
            },
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: Some(Utc::now()), // Already deleted
            },
        ];

        let op = SoftDeleteOperation::new();
        let count = op.soft_delete_batch(&mut entities).unwrap();

        assert_eq!(count, 2); // Only 2 were not deleted
        assert!(entities.iter().all(|e| e.is_deleted()));
    }

    #[test]
    fn test_batch_restore() {
        let mut entities = vec![
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: Some(Utc::now()),
            },
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: Some(Utc::now()),
            },
            TestEntity {
                id: Uuid::new_v4(),
                deleted_at: None, // Not deleted
            },
        ];

        let op = SoftDeleteOperation::new();
        let count = op.restore_batch(&mut entities).unwrap();

        assert_eq!(count, 2); // Only 2 were deleted
        assert!(entities.iter().all(|e| !e.is_deleted()));
    }
}
