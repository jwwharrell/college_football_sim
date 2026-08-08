//! sim_core: Pure domain logic for a deterministic college football simulator.
//!
//! Principles:
//! - No I/O or persistence here (pure, testable functions).
//! - Deterministic via seeded RNG (injected at the edges).
//! - Explicit errors via thiserror; no unwraps in app code.
//! - Player rosters are an adjacent dynasty-domain model and do not alter the
//!   aggregate [`team::Team`] inputs consumed by possession simulation.

pub mod calibration;
pub mod depth_chart;
pub mod game;
pub mod player;
pub mod rng;
pub mod roster;
pub mod season;
pub mod simulation;
pub mod team;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// Crate-global result type for domain operations.
pub type SimResult<T> = Result<T, SimError>;

/// Domain error type for the simulator core.
#[derive(Debug, Error)]
pub enum SimError {
    /// Error when a required team is not found
    #[error("team not found: {0}")]
    TeamNotFound(String),

    /// Error when a required game is not found
    #[error("game not found: {0}")]
    GameNotFound(String),

    /// Error when a simulation operation fails
    #[error("simulation error: {0}")]
    SimulationError(String),

    /// Error when an invalid parameter is provided
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// Error when an operation is attempted on a game with an invalid status
    #[error("invalid game status for operation")]
    InvalidGameStatus,

    /// Error when a regular-season schedule violates domain invariants.
    #[error("invalid schedule: {0}")]
    InvalidSchedule(String),

    /// Error when serialized or in-memory season state cannot progress safely.
    #[error("invalid season state: {0}")]
    InvalidSeasonState(String),

    /// Error when progression is requested after the final week.
    #[error("regular season is complete")]
    SeasonComplete,

    /// Placeholder for other errors
    #[error("other error: {0}")]
    Other(String),
}

/// Trivial function to prove the library links; replace with real APIs incrementally.
pub fn ping() -> &'static str {
    "sim_core"
}

/// Version of the sim_core crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
