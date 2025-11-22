//! Test setup helper
//!
//! This module loads environment variables from .env file for tests

pub fn setup() {
    let _ = dotenvy::dotenv();
}
