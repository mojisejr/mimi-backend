//! Test setup helper
//!
//! This module loads environment variables from .env file for tests

#[allow(dead_code)]
pub fn setup() {
    let _ = dotenvy::dotenv();
}
