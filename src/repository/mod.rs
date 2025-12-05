//! Repository Module
//!
//! Database repository patterns for MimiVibe Backend.
//! Provides soft delete functionality, deduplication, and query helpers.

pub mod dedupe;
pub mod job_repository;
pub mod prompt_repository;
pub mod soft_delete;

pub use job_repository::*;
pub use prompt_repository::{PromptRepository, PromptRepositoryError};
