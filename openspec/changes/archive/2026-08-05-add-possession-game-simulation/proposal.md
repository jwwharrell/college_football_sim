## Why

The simulator can represent teams, games, scores, and seasons, but it cannot yet produce a game outcome from team quality. A deterministic, statistically calibrated simulation engine is the foundation needed for credible long-running dynasty gameplay, where management decisions must create explainable effects over many games and seasons.

## What Changes

- Add a possession-level game simulator that converts the current offense, defense, special-teams, and overall team ratings into drives, scoring, turnovers, aggregate statistics, and a completed game result.
- Make every simulation reproducible from explicit inputs, a versioned configuration, and a seed.
- Model home-field advantage, neutral sites, regulation, and overtime without permitting completed games to remain tied.
- Return a structured game summary suitable for CLI output and future match reports rather than mutating persistence or performing I/O in the simulation core.
- Add configurable calibration parameters so statistical behavior can evolve independently of simulation mechanics.
- Add deterministic batch simulation and aggregate validation against documented statistical envelopes, including scoring, possessions, turnovers, home advantage, favorite performance, upset rate, and overtime frequency.
- Expose a CLI command that runs a rated matchup with a seed and prints its result and statistical summary.

## Capabilities

### New Capabilities

- `possession-game-simulation`: Deterministic possession-level simulation of a rated matchup, including drives, regulation and overtime results, and structured team/game statistics.
- `simulation-calibration`: Versioned tuning parameters and reproducible batch analysis that validate aggregate game behavior against realistic statistical envelopes.

### Modified Capabilities

None.

## Impact

- `sim_core` gains pure simulation inputs, configuration, possession outcomes, statistical summaries, and orchestration APIs that use the existing `Team`, `Game`, and `SimRng` types.
- The existing game lifecycle must accommodate simulation-produced quarter scores and overtime while preserving current state-transition validation.
- `cli` gains a user-facing simulation harness and optional batch/calibration reporting; it remains responsible for parsing and presentation.
- `persistence` is not required for this change, and no storage format or database schema changes are proposed.
- Automated tests expand from example behavior to seeded snapshots, invariants, rating monotonicity checks, and reproducible aggregate statistical validation.
