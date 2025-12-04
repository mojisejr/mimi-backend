//! Data Models and Structures
//!
//! Defines request/response models and domain types for the MimiVibe backend.

// pub mod reading_job; // Temporarily excluded due to database schema issues
pub mod job_types;
pub mod prompt;
pub mod reading_job_simple;
pub mod tarot_request;

// Use specific exports to avoid conflicts
// Note: reading_job is temporarily excluded due to database schema issues
// pub use reading_job::{
//     ReadingJob, CreateJobInput, UpdateJobStatusInput, JobStatus, JobStatusError,
//     JobMetadata, JobRetryInfo, JobError, JobQueryBuilder
// };
// Use selective imports to avoid ambiguous re-exports
pub use job_types::*;
pub use tarot_request::*;

// Re-export specific types with qualified names to avoid conflicts
pub use job_types::{
    CreateJobInput as CreateJobInputType, JobStatus as JobStatusType, ReadingJob as ReadingJobType,
};
