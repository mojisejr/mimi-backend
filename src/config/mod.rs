//! Configuration module for MimiVibe backend
//!
//! This module provides environment-driven configuration for the application,
//! including queue connection pool settings, retry policies, and environment-specific
//! parameters.

pub mod env;

pub use env::{Environment, EnvironmentConfig, QueuePoolConfig};
