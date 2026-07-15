//! persistence: Data access layer for the simulator.
//!
//! Responsibilities:
//! - Provide adapters to load/save domain DTOs (e.g., teams, schedules).
//! - Keep I/O and storage concerns out of sim_core.
//!
//! This crate should not contain simulation logic.

use thiserror::Error;

/// Crate-global result type for persistence operations.
pub type PersistResult<T> = Result<T, PersistError>;

/// Error type for persistence layer operations.
#[derive(Debug, Error)]
pub enum PersistError {
    /// Placeholder until real storage backends are implemented.
    #[error("not implemented")]
    NotImplemented,
}

/// Trivial function to prove the library links; replace with real storage APIs.
pub fn ping() -> &'static str {
    "persistence"
}
