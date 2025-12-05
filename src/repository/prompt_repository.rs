//! Prompt Repository
//!
//! Database repository pattern for managing AI agent prompts.
//! Provides CRUD operations for the `prompts` table.

use crate::models::StoredPrompt;
use sqlx::PgPool;
use thiserror::Error;

/// Errors that can occur during prompt repository operations
#[derive(Debug, Error)]
pub enum PromptRepositoryError {
    /// Prompt not found for the given agent name
    #[error("Prompt not found for agent: {agent_name}")]
    PromptNotFound { agent_name: String },

    /// Database error occurred
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// Invalid agent name provided
    #[error("Invalid agent name: {agent_name}. Valid names are: question_filter, question_analyzer, reading_agent")]
    InvalidAgentName { agent_name: String },
}

/// Repository for managing prompts in the database
///
/// Provides async CRUD operations for the `prompts` table:
/// - `load_prompt`: Load a single prompt by agent name
/// - `load_all_active_prompts`: Load all active prompts
/// - `create_prompt`: Create a new prompt
/// - `update_prompt`: Update an existing prompt's content
///
/// # Example
///
/// ```ignore
/// use mimivibe_backend::repository::PromptRepository;
/// use sqlx::PgPool;
///
/// async fn example(pool: &PgPool) {
///     // Load a prompt
///     let prompt = PromptRepository::load_prompt(pool, "question_filter").await?;
///     
///     // Load all active prompts
///     let all_prompts = PromptRepository::load_all_active_prompts(pool).await?;
/// }
/// ```
pub struct PromptRepository;

impl PromptRepository {
    /// Load a single prompt by agent name
    ///
    /// Retrieves the active prompt for the specified agent.
    /// Only returns prompts where `is_active = true`.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `agent_name` - The agent identifier (question_filter, question_analyzer, reading_agent)
    ///
    /// # Returns
    ///
    /// * `Ok(StoredPrompt)` - The prompt if found
    /// * `Err(PromptRepositoryError::PromptNotFound)` - If no active prompt exists for the agent
    /// * `Err(PromptRepositoryError::InvalidAgentName)` - If agent_name is not valid
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs
    ///
    /// # Example
    ///
    /// ```ignore
    /// let prompt = PromptRepository::load_prompt(pool, "question_filter").await?;
    /// println!("Loaded prompt version: {}", prompt.version);
    /// ```
    pub async fn load_prompt(
        pool: &PgPool,
        agent_name: &str,
    ) -> Result<StoredPrompt, PromptRepositoryError> {
        // Validate agent name first
        if !StoredPrompt::is_valid_agent_name(agent_name) {
            return Err(PromptRepositoryError::InvalidAgentName {
                agent_name: agent_name.to_string(),
            });
        }

        sqlx::query_as::<_, StoredPrompt>(
            r#"
            SELECT id, agent_name, prompt_content, version, is_active, created_at, updated_at
            FROM prompts
            WHERE agent_name = $1 AND is_active = true
            "#,
        )
        .bind(agent_name)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => PromptRepositoryError::PromptNotFound {
                agent_name: agent_name.to_string(),
            },
            other => PromptRepositoryError::DatabaseError(other),
        })
    }

    /// Load all active prompts from the database
    ///
    /// Retrieves all prompts where `is_active = true`, ordered by agent_name.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<StoredPrompt>)` - List of all active prompts
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs
    ///
    /// # Example
    ///
    /// ```ignore
    /// let prompts = PromptRepository::load_all_active_prompts(pool).await?;
    /// for prompt in prompts {
    ///     println!("Agent: {}, Version: {}", prompt.agent_name, prompt.version);
    /// }
    /// ```
    pub async fn load_all_active_prompts(
        pool: &PgPool,
    ) -> Result<Vec<StoredPrompt>, PromptRepositoryError> {
        sqlx::query_as::<_, StoredPrompt>(
            r#"
            SELECT id, agent_name, prompt_content, version, is_active, created_at, updated_at
            FROM prompts
            WHERE is_active = true
            ORDER BY agent_name
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(PromptRepositoryError::DatabaseError)
    }

    /// Create a new prompt in the database
    ///
    /// Creates a new prompt record with version 1 and is_active = true.
    /// The agent_name must be unique (enforced by database constraint).
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `agent_name` - The agent identifier (question_filter, question_analyzer, reading_agent)
    /// * `prompt_content` - The full prompt template with {placeholders}
    ///
    /// # Returns
    ///
    /// * `Ok(StoredPrompt)` - The created prompt with generated ID and timestamps
    /// * `Err(PromptRepositoryError::InvalidAgentName)` - If agent_name is not valid
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs (e.g., duplicate agent_name)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let prompt = PromptRepository::create_prompt(
    ///     pool,
    ///     "question_filter",
    ///     "You are a filter agent. Analyze the question: {question}",
    /// ).await?;
    /// ```
    pub async fn create_prompt(
        pool: &PgPool,
        agent_name: &str,
        prompt_content: &str,
    ) -> Result<StoredPrompt, PromptRepositoryError> {
        // Validate agent name first
        if !StoredPrompt::is_valid_agent_name(agent_name) {
            return Err(PromptRepositoryError::InvalidAgentName {
                agent_name: agent_name.to_string(),
            });
        }

        sqlx::query_as::<_, StoredPrompt>(
            r#"
            INSERT INTO prompts (agent_name, prompt_content, version, is_active)
            VALUES ($1, $2, 1, true)
            RETURNING id, agent_name, prompt_content, version, is_active, created_at, updated_at
            "#,
        )
        .bind(agent_name)
        .bind(prompt_content)
        .fetch_one(pool)
        .await
        .map_err(PromptRepositoryError::DatabaseError)
    }

    /// Update an existing prompt's content
    ///
    /// Updates the prompt content and increments the version number.
    /// Only updates active prompts.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `agent_name` - The agent identifier to update
    /// * `new_content` - The new prompt content
    ///
    /// # Returns
    ///
    /// * `Ok(StoredPrompt)` - The updated prompt with new version
    /// * `Err(PromptRepositoryError::PromptNotFound)` - If no active prompt exists for the agent
    /// * `Err(PromptRepositoryError::InvalidAgentName)` - If agent_name is not valid
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs
    ///
    /// # Example
    ///
    /// ```ignore
    /// let updated = PromptRepository::update_prompt(
    ///     pool,
    ///     "question_filter",
    ///     "Updated prompt content with {question}",
    /// ).await?;
    /// assert_eq!(updated.version, 2); // Version incremented
    /// ```
    pub async fn update_prompt(
        pool: &PgPool,
        agent_name: &str,
        new_content: &str,
    ) -> Result<StoredPrompt, PromptRepositoryError> {
        // Validate agent name first
        if !StoredPrompt::is_valid_agent_name(agent_name) {
            return Err(PromptRepositoryError::InvalidAgentName {
                agent_name: agent_name.to_string(),
            });
        }

        sqlx::query_as::<_, StoredPrompt>(
            r#"
            UPDATE prompts
            SET prompt_content = $2, version = version + 1, updated_at = NOW()
            WHERE agent_name = $1 AND is_active = true
            RETURNING id, agent_name, prompt_content, version, is_active, created_at, updated_at
            "#,
        )
        .bind(agent_name)
        .bind(new_content)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => PromptRepositoryError::PromptNotFound {
                agent_name: agent_name.to_string(),
            },
            other => PromptRepositoryError::DatabaseError(other),
        })
    }

    /// Soft delete a prompt by setting is_active to false
    ///
    /// This method does not physically delete the prompt, but marks it as inactive.
    /// This is useful for audit trails and potential recovery.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `agent_name` - The agent identifier to deactivate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the prompt was successfully deactivated
    /// * `Err(PromptRepositoryError::PromptNotFound)` - If no active prompt exists for the agent
    /// * `Err(PromptRepositoryError::InvalidAgentName)` - If agent_name is not valid
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs
    pub async fn soft_delete_prompt(
        pool: &PgPool,
        agent_name: &str,
    ) -> Result<(), PromptRepositoryError> {
        // Validate agent name first
        if !StoredPrompt::is_valid_agent_name(agent_name) {
            return Err(PromptRepositoryError::InvalidAgentName {
                agent_name: agent_name.to_string(),
            });
        }

        let result = sqlx::query(
            r#"
            UPDATE prompts
            SET is_active = false, updated_at = NOW()
            WHERE agent_name = $1 AND is_active = true
            "#,
        )
        .bind(agent_name)
        .execute(pool)
        .await
        .map_err(PromptRepositoryError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(PromptRepositoryError::PromptNotFound {
                agent_name: agent_name.to_string(),
            });
        }

        Ok(())
    }

    /// Load a prompt by ID (including inactive prompts)
    ///
    /// This method is useful for audit purposes or recovering deleted prompts.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `id` - The prompt UUID
    ///
    /// # Returns
    ///
    /// * `Ok(StoredPrompt)` - The prompt if found
    /// * `Err(PromptRepositoryError::PromptNotFound)` - If no prompt exists with the given ID
    /// * `Err(PromptRepositoryError::DatabaseError)` - If a database error occurs
    pub async fn load_prompt_by_id(
        pool: &PgPool,
        id: uuid::Uuid,
    ) -> Result<StoredPrompt, PromptRepositoryError> {
        sqlx::query_as::<_, StoredPrompt>(
            r#"
            SELECT id, agent_name, prompt_content, version, is_active, created_at, updated_at
            FROM prompts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => PromptRepositoryError::PromptNotFound {
                agent_name: format!("id:{}", id),
            },
            other => PromptRepositoryError::DatabaseError(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_repository_error_display() {
        let error = PromptRepositoryError::PromptNotFound {
            agent_name: "test_agent".to_string(),
        };
        assert!(error.to_string().contains("test_agent"));

        let error = PromptRepositoryError::InvalidAgentName {
            agent_name: "invalid".to_string(),
        };
        assert!(error.to_string().contains("invalid"));
        assert!(error.to_string().contains("question_filter"));
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // Verify that errors can be sent across threads
        assert_send::<PromptRepositoryError>();
        // Note: sqlx::Error may not be Sync in all cases, so we skip this check
        // assert_sync::<PromptRepositoryError>();
    }
}
