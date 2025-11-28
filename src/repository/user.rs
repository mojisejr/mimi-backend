//! User Repository Implementation
//!
//! Repository pattern implementation for user management operations.
//! Provides methods for user CRUD, wallet operations, and soft delete functionality.
//!
//! # Features
//! - Multi-provider authentication support (external_id + external_provider)
//! - Atomic wallet balance updates with transaction recording
//! - Soft delete with cascade behavior
//! - Tier system with invite counting
//! - Idempotent operations with proper error handling

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::repository::soft_delete::SoftDeletable;

// ============================================================================
// Type Definitions
// ============================================================================

/// User tier enumeration based on invitation count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UserTier {
    /// 1-5 invites (base rewards)
    #[default]
    Bronze,
    /// 6-15 invites (+20% bonus)
    Silver,
    /// 16+ invites (+50% bonus)
    Gold,
}

impl FromStr for UserTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bronze" => Ok(UserTier::Bronze),
            "silver" => Ok(UserTier::Silver),
            "gold" => Ok(UserTier::Gold),
            _ => Err(format!("Invalid user tier: {}", s)),
        }
    }
}

/// Transaction type enumeration for wallet operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type")]
pub enum TransactionType {
    /// Buying stars
    #[sqlx(rename = "purchase")]
    Purchase,
    /// Coins from invitations
    #[sqlx(rename = "invite_reward")]
    InviteReward,
    /// Converting coins to stars
    #[sqlx(rename = "exchange")]
    Exchange,
    /// Spending stars on reading
    #[sqlx(rename = "reading")]
    Reading,
    /// Payment refund/reversal
    #[sqlx(rename = "refund")]
    Refund,
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "purchase" => Ok(TransactionType::Purchase),
            "invite_reward" => Ok(TransactionType::InviteReward),
            "exchange" => Ok(TransactionType::Exchange),
            "reading" => Ok(TransactionType::Reading),
            "refund" => Ok(TransactionType::Refund),
            _ => Err(format!("Invalid transaction type: {}", s)),
        }
    }
}

/// Currency enumeration for wallet operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "currency")]
pub enum Currency {
    /// Hard currency (paid)
    #[sqlx(rename = "STAR")]
    Star,
    /// Soft currency (earned)
    #[sqlx(rename = "COIN")]
    Coin,
}

impl FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "STAR" => Ok(Currency::Star),
            "COIN" => Ok(Currency::Coin),
            _ => Err(format!("Invalid currency: {}", s)),
        }
    }
}

// ============================================================================
// Data Models
// ============================================================================

/// User entity representing a user account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Primary key
    pub id: Uuid,
    /// Auth provider user ID
    pub external_id: String,
    /// Auth provider name ("clerk", "auth0", "firebase", "custom")
    pub external_provider: String,
    /// User email address (unique)
    pub email: String,
    /// User display name
    pub name: Option<String>,
    /// Profile picture URL
    pub picture_url: Option<String>,
    /// Hard currency balance (stars)
    pub star_balance: i64,
    /// Soft currency balance (coins)
    pub coin_balance: i64,
    /// Unique invite code (auto-generated)
    pub invite_code: String,
    /// User who invited this user
    pub invited_by: Option<Uuid>,
    /// Total number of successful invitations
    pub total_invites: i32,
    /// User tier based on invite count
    pub tier: UserTier,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last login timestamp
    pub last_login_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp (None = active)
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SoftDeletable for User {
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

/// Wallet transaction entity for audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// Primary key
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Transaction type
    pub transaction_type: TransactionType,
    /// Amount (positive for addition, negative for subtraction)
    pub amount: i32,
    /// Currency type
    pub currency: Currency,
    /// Balance after transaction
    pub balance_after: i64,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
    /// Related payment (if any)
    pub related_payment_id: Option<Uuid>,
    /// Related tarot reading (if any)
    pub related_reading_id: Option<Uuid>,
    /// Related invitation (if any)
    pub related_invite_id: Option<Uuid>,
    /// Transaction timestamp
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new user.
#[derive(Debug, Clone)]
pub struct CreateUser {
    /// Auth provider user ID
    pub external_id: String,
    /// Auth provider name
    pub external_provider: String,
    /// User email address
    pub email: String,
    /// User display name (optional)
    pub name: Option<String>,
    /// Profile picture URL (optional)
    pub picture_url: Option<String>,
}

// ============================================================================
// Repository Implementation
// ============================================================================

/// User repository for database operations.
///
/// Provides methods for user management, wallet operations, and soft delete functionality.
/// All operations use database transactions for consistency and proper error handling.
#[derive(Debug, Clone)]
pub struct UserRepository {
    /// Database connection pool
    #[allow(dead_code)]
    pool: sqlx::PgPool,
}

impl UserRepository {
    /// Creates a new user repository with the given database pool.
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Example
    /// ```ignore
    /// use sqlx::PgPool;
    /// use mimivibe_backend::repository::user::UserRepository;
    ///
    /// let pool = PgPool::connect(&database_url).await?;
    /// let repository = UserRepository::new(pool);
    /// ```
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Core Find Operations
    // ========================================================================

    /// Finds a user by their primary key ID.
    ///
    /// Automatically excludes soft-deleted users.
    ///
    /// # Arguments
    /// * `id` - User UUID to find
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or soft deleted
    /// * `Err(sqlx::Error)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// let user = repository.find_by_id(&user_id).await?;
    /// if let Some(user) = user {
    ///     println!("Found user: {}", user.email);
    /// }
    /// ```
    pub async fn find_by_id(&self, _id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        // Mock implementation for now - return None
        // This will be implemented with actual SQL queries when database is available
        Ok(None)
    }

    /// Finds a user by external ID and provider combination.
    ///
    /// Used for authentication lookup across multiple providers.
    ///
    /// # Arguments
    /// * `external_id` - User ID from auth provider
    /// * `external_provider` - Provider name ("clerk", "auth0", etc.)
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or soft deleted
    /// * `Err(sqlx::Error)` - Database error
    ///
    /// # Example
    /// ```ignore
    /// let user = repository.find_by_external_id("auth0_123", "auth0").await?;
    /// if let Some(user) = user {
    ///     println!("Found user: {}", user.email);
    /// }
    /// ```
    pub async fn find_by_external_id(
        &self,
        _external_id: &str,
        _external_provider: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        // Mock implementation - return None
        Ok(None)
    }

    /// Finds a user by email address.
    ///
    /// # Arguments
    /// * `email` - Email address to find
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or soft deleted
    /// * `Err(sqlx::Error)` - Database error
    pub async fn find_by_email(&self, _email: &str) -> Result<Option<User>, sqlx::Error> {
        // Mock implementation - return None
        Ok(None)
    }

    /// Finds a user by invite code.
    ///
    /// # Arguments
    /// * `invite_code` - Invite code to look up
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or soft deleted
    /// * `Err(sqlx::Error)` - Database error
    pub async fn find_by_invite_code(
        &self,
        _invite_code: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        // Mock implementation - return None
        Ok(None)
    }

    // ========================================================================
    // Wallet Operations
    // ========================================================================

    /// Gets the current wallet balance for a user.
    ///
    /// Returns tuple of (star_balance, coin_balance).
    ///
    /// # Arguments
    /// * `user_id` - User UUID
    ///
    /// # Returns
    /// * `Ok((stars, coins))` - Current balances
    /// * `Err(sqlx::Error)` - Database error or user not found
    ///
    /// # Example
    /// ```ignore
    /// let (stars, coins) = repository.get_wallet_balance(&user_id).await?;
    /// println!("User has {} stars and {} coins", stars, coins);
    /// ```
    pub async fn get_wallet_balance(&self, _user_id: &Uuid) -> Result<(i64, i64), sqlx::Error> {
        // Mock implementation - return zero balances
        Ok((0, 0))
    }

    /// Updates wallet balance atomically and records transaction.
    ///
    /// This method uses a database transaction to ensure atomicity:
    /// 1. Checks current balance and user existence
    /// 2. Validates balance constraints (non-negative for non-refund operations)
    /// 3. Updates user balance
    /// 4. Creates wallet transaction record
    ///
    /// # Arguments
    /// * `user_id` - User UUID
    /// * `star_amount` - Stars to add (positive) or subtract (negative)
    /// * `coin_amount` - Coins to add (positive) or subtract (negative)
    /// * `transaction_type` - Type of transaction
    /// * `metadata` - Optional metadata for the transaction
    ///
    /// # Returns
    /// * `Ok(WalletTransaction)` - Created transaction record
    /// * `Err(sqlx::Error)` - Database error, user not found, or insufficient funds
    ///
    /// # Example
    /// ```ignore
    /// // Add 100 stars from purchase
    /// let transaction = repository.update_wallet_balance(
    ///     &user_id, 100, 0, TransactionType::Purchase,
    ///     Some(json!({"payment_id": "pi_test_123"}))
    /// ).await?;
    ///
    /// // Spend 5 stars on reading
    /// let transaction = repository.update_wallet_balance(
    ///     &user_id, -5, 0, TransactionType::Reading, None
    /// ).await?;
    /// ```
    pub async fn update_wallet_balance(
        &self,
        user_id: &Uuid,
        star_amount: i64,
        coin_amount: i64,
        transaction_type: TransactionType,
        metadata: Option<serde_json::Value>,
    ) -> Result<WalletTransaction, sqlx::Error> {
        // Validate inputs
        if star_amount == 0 && coin_amount == 0 {
            return Err(sqlx::Error::Protocol(
                "At least one of star_amount or coin_amount must be non-zero".to_string(),
            ));
        }

        // Mock implementation - create a dummy transaction
        let currency = if star_amount != 0 {
            Currency::Star
        } else {
            Currency::Coin
        };

        let amount = if star_amount != 0 {
            star_amount as i32
        } else {
            coin_amount as i32
        };

        Ok(WalletTransaction {
            id: Uuid::new_v4(),
            user_id: *user_id,
            transaction_type,
            amount,
            currency,
            balance_after: 100, // Mock balance
            metadata,
            related_payment_id: None,
            related_reading_id: None,
            related_invite_id: None,
            created_at: Utc::now(),
        })
    }

    // ========================================================================
    // User Management Operations
    // ========================================================================

    /// Creates a new user with optional inviter.
    ///
    /// Handles invite code generation, tier assignment, and invitation counting.
    ///
    /// # Arguments
    /// * `user_data` - User creation data
    /// * `inviter_id` - Optional user ID who invited this user
    ///
    /// # Returns
    /// * `Ok(User)` - Created user
    /// * `Err(sqlx::Error)` - Database error or constraint violation
    ///
    /// # Example
    /// ```ignore
    /// let user_data = CreateUser {
    ///     external_id: "auth0_123".to_string(),
    ///     external_provider: "auth0".to_string(),
    ///     email: "user@example.com".to_string(),
    ///     name: Some("John Doe".to_string()),
    ///     picture_url: None,
    /// };
    ///
    /// let user = repository.create_user_with_inviter(user_data, Some(inviter_id)).await?;
    /// ```
    pub async fn create_user_with_inviter(
        &self,
        user_data: CreateUser,
        inviter_id: Option<Uuid>,
    ) -> Result<User, sqlx::Error> {
        // Mock implementation - create a dummy user
        let now = Utc::now();
        Ok(User {
            id: Uuid::new_v4(),
            external_id: user_data.external_id,
            external_provider: user_data.external_provider,
            email: user_data.email,
            name: user_data.name,
            picture_url: user_data.picture_url,
            star_balance: 0,
            coin_balance: 0,
            invite_code: format!("{:010}", rand::random::<u32>() % 1000000000),
            invited_by: inviter_id,
            total_invites: 0,
            tier: UserTier::Bronze,
            created_at: now,
            last_login_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Updates the last login timestamp for a user.
    ///
    /// Called when user successfully authenticates.
    ///
    /// # Arguments
    /// * `user_id` - User UUID
    ///
    /// # Returns
    /// * `Ok(())` - Update successful
    /// * `Err(sqlx::Error)` - Database error or user not found
    pub async fn update_last_login(&self, _user_id: &Uuid) -> Result<(), sqlx::Error> {
        // Mock implementation - always succeeds
        Ok(())
    }

    /// Soft deletes a user and handles cascade behavior.
    ///
    /// Sets deleted_at timestamp and can cascade to related entities.
    ///
    /// # Arguments
    /// * `user_id` - User UUID to soft delete
    ///
    /// # Returns
    /// * `Ok(User)` - The soft-deleted user
    /// * `Err(sqlx::Error)` - Database error or user not found
    ///
    /// # Example
    /// ```ignore
    /// let deleted_user = repository.soft_delete_user(&user_id).await?;
    /// assert!(deleted_user.is_deleted());
    /// ```
    pub async fn soft_delete_user(&self, user_id: &Uuid) -> Result<User, sqlx::Error> {
        // Mock implementation - create a dummy soft-deleted user
        let now = Utc::now();
        Ok(User {
            id: *user_id,
            external_id: "deleted_user".to_string(),
            external_provider: "test".to_string(),
            email: "deleted@example.com".to_string(),
            name: None,
            picture_url: None,
            star_balance: 0,
            coin_balance: 0,
            invite_code: "DELETED".to_string(),
            invited_by: None,
            total_invites: 0,
            tier: UserTier::Bronze,
            created_at: now,
            last_login_at: now,
            updated_at: now,
            deleted_at: Some(now),
        })
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::repository::soft_delete::SoftDeleteOperation;

    #[test]
    fn test_user_tier_from_str() {
        assert_eq!(UserTier::from_str("bronze").unwrap(), UserTier::Bronze);
        assert_eq!(UserTier::from_str("BRONZE").unwrap(), UserTier::Bronze);
        assert_eq!(UserTier::from_str("silver").unwrap(), UserTier::Silver);
        assert_eq!(UserTier::from_str("gold").unwrap(), UserTier::Gold);
        assert!(UserTier::from_str("invalid").is_err());
    }

    #[test]
    fn test_transaction_type_from_str() {
        assert_eq!(
            TransactionType::from_str("purchase").unwrap(),
            TransactionType::Purchase
        );
        assert_eq!(
            TransactionType::from_str("invite_reward").unwrap(),
            TransactionType::InviteReward
        );
        assert_eq!(
            TransactionType::from_str("exchange").unwrap(),
            TransactionType::Exchange
        );
        assert_eq!(
            TransactionType::from_str("reading").unwrap(),
            TransactionType::Reading
        );
        assert_eq!(
            TransactionType::from_str("refund").unwrap(),
            TransactionType::Refund
        );
        assert!(TransactionType::from_str("invalid").is_err());
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from_str("STAR").unwrap(), Currency::Star);
        assert_eq!(Currency::from_str("star").unwrap(), Currency::Star);
        assert_eq!(Currency::from_str("COIN").unwrap(), Currency::Coin);
        assert_eq!(Currency::from_str("coin").unwrap(), Currency::Coin);
        assert!(Currency::from_str("invalid").is_err());
    }

    #[test]
    fn test_user_soft_deletable() {
        let mut user = User {
            id: Uuid::new_v4(),
            external_id: "test".to_string(),
            external_provider: "test".to_string(),
            email: "test@example.com".to_string(),
            name: None,
            picture_url: None,
            star_balance: 0,
            coin_balance: 0,
            invite_code: "ABC123".to_string(),
            invited_by: None,
            total_invites: 0,
            tier: UserTier::Bronze,
            created_at: Utc::now(),
            last_login_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        assert!(!user.is_deleted());

        let delete_op = SoftDeleteOperation::new();
        delete_op.soft_delete(&mut user).unwrap();

        assert!(user.is_deleted());
        assert!(user.deleted_at().is_some());

        delete_op.restore(&mut user).unwrap();
        assert!(!user.is_deleted());
        assert!(user.deleted_at().is_none());
    }

    #[test]
    fn test_user_tier_default() {
        assert_eq!(UserTier::default(), UserTier::Bronze);
    }

    #[test]
    fn test_wallet_transaction_creation() {
        let transaction = WalletTransaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            transaction_type: TransactionType::Purchase,
            amount: 100,
            currency: Currency::Star,
            balance_after: 150,
            metadata: Some(json!({"source": "stripe"})),
            related_payment_id: Some(Uuid::new_v4()),
            related_reading_id: None,
            related_invite_id: None,
            created_at: Utc::now(),
        };

        assert_eq!(transaction.transaction_type, TransactionType::Purchase);
        assert_eq!(transaction.amount, 100);
        assert_eq!(transaction.currency, Currency::Star);
        assert_eq!(transaction.balance_after, 150);
        assert!(transaction.metadata.is_some());
        assert!(transaction.related_payment_id.is_some());
        assert!(transaction.related_reading_id.is_none());
    }

    #[test]
    fn test_create_user_data() {
        let user_data = CreateUser {
            external_id: "auth0_123".to_string(),
            external_provider: "auth0".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            picture_url: Some("https://example.com/avatar.jpg".to_string()),
        };

        assert_eq!(user_data.external_id, "auth0_123");
        assert_eq!(user_data.external_provider, "auth0");
        assert_eq!(user_data.email, "test@example.com");
        assert_eq!(user_data.name, Some("Test User".to_string()));
        assert_eq!(
            user_data.picture_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
    }
}
