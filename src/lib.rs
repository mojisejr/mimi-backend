//! MimiVibe Backend - Shared Library
//!
//! This crate provides shared functionality for the MimiVibe backend monorepo.
//! It includes modules for agents, API routes, authentication, models, and utilities.

pub mod agents;
pub mod api;
pub mod auth;
pub mod models;
pub mod queue;
pub mod utils;
pub mod worker;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
