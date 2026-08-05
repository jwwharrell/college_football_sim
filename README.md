# College Football Simulator (Deterministic, Testable)

This workspace hosts a deterministic, testable college football simulator with a CLI today and a pluggable GUI later.

Principles
- Separate domain (sim_core) from persistence (persistence) and UI (cli).
- Keep simulation pure (no I/O or DB calls in sim_core).
- Deterministic simulation with seeded RNG.
- Explicit errors with thiserror; validated construction and state transitions return errors.
- Prefer composition; keep modules small and documented.
- Add unit tests for core logic; maintain Clippy cleanliness and rustfmt formatting.

Workspace layout
- crates/sim_core: library crate with pure domain logic (no I/O).
- crates/persistence: library crate with data access adapters (e.g., SQLite/JSON).
- crates/cli: binary crate that wires inputs, seeds RNG, and prints results.

Tooling and dependencies
- Workspace-managed dependencies:
  - anyhow, thiserror, serde (+derive), rand, rand_chacha, tracing, tracing-subscriber.
- Crate-specific:
  - persistence: rusqlite = { version = "0.31", features=["bundled","chrono"] }.
  - cli: clap = { version="4", features=["derive"] }, inquire = "0.7".

Build
- Build all crates:
  - cargo build
- Run the CLI:
  - `cargo run -p cli -- --help`

CLI feature harness
- Check that workspace components are linked:
  - `cargo run -p cli -- health`
- Inspect a deterministic RNG sequence:
  - `cargo run -p cli -- rng --seed 42 --count 5 --max 10`
- Exercise game scoring and winner selection:
  - `cargo run -p cli -- game --home-score 24 --away-score 17 --conference`
- Exercise season record calculation:
  - `cargo run -p cli -- season --home-score 10 --away-score 20`
- Simulate a deterministic possession-level rated matchup:
  - `cargo run -p cli -- simulate --seed 42 --home-rating 82 --home-offense 85 --home-defense 80 --home-special-teams 76 --away-rating 75 --away-offense 78 --away-defense 74 --away-special-teams 72`
  - Add `--neutral` to remove the configured home-field modifier.
- Run the canonical aggregate calibration suite:
  - `cargo run -p cli -- calibrate --seeds 1000`
  - Add `--json` for a machine-readable report. A failed statistical envelope produces a non-zero exit.

Determinism
- All randomness flows through a seedable RNG (rand_chacha::ChaCha8/ChaCha20 as needed).
- The CLI will pass a --seed u64 to the simulator to guarantee reproducible outcomes.
- Exact game replay requires identical matchup inputs, seed, simulation algorithm version, and calibration profile version. Results report all four values needed for provenance.
- Domain-specific seeds use a stable FNV-1a/SplitMix64 derivation so regulation and overtime random streams remain isolated.

Simulation model
- The initial engine operates at possession resolution. It produces drives, period and overtime scoring, field-position transitions, and reconciled team box scores.
- Current offense and defense ratings drive matchup efficiency, overall rating adds a bounded consistency effect, special-teams rating affects relevant efficiency and field position, and non-neutral home teams receive a configurable advantage.
- The public `MatchupModifiers` seam is reserved for future roster units, schemes, coaching, fatigue, weather, and injury effects. Those dynasty systems are intentionally not modeled yet.
- The current overtime model is a simplified, versioned college format rather than a complete set of historical NCAA rules.
- See `CALIBRATION.md` for statistical targets, the checked-in baseline, runtime, and profile update process.

Error handling
- Errors are explicit and bubble up using thiserror-based enums.
- No unwrap in application code; error contexts use anyhow at the CLI boundary when appropriate.

Separation of concerns
- sim_core exposes pure functions and data types; no persistence calls or global state.
- persistence provides trait-based storage backends (e.g., rusqlite, JSON).
- cli orchestrates: parses args, loads data through persistence, seeds RNG, calls sim_core, prints results.

Status
- Workspace scaffold with Cargo manifests and crate entry files.
- Core domain types (Team, Game, Season), deterministic ChaCha8 RNG, and validation defined in sim_core.
- Unit tests cover deterministic RNG, scoring, records, lifecycle transitions, and invalid inputs.
- Next steps:
  - Introduce storage trait(s) and a simple JSON adapter in persistence.
  - Add CLI subcommands to import data and run week/season simulations.
  - Add unit tests in sim_core to enforce determinism and model behavior.

License
- TBD.
