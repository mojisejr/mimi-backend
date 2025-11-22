//! Worker module for background job processing
//!
//! This module provides functionality for worker processes that handle
//! asynchronous job processing with retry logic and error handling.

pub mod retry;

pub use retry::{RetryConfig, RetryPolicy};
