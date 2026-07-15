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
  - cargo run -p cli -- -v

Determinism
- All randomness flows through a seedable RNG (rand_chacha::ChaCha8/ChaCha20 as needed).
- The CLI will pass a --seed u64 to the simulator to guarantee reproducible outcomes.

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
