## 1. Simulation Contracts and Configuration

- [x] 1.1 Add simulation module boundaries and public input, matchup, venue, algorithm-version, and result-provenance types in `sim_core`.
- [x] 1.2 Define the serializable versioned calibration profile, including drive, outcome, clock, rating, venue, overtime, numeric-bound, and acceptance-envelope parameters.
- [x] 1.3 Implement exhaustive profile and matchup validation with descriptive domain errors for non-finite values, invalid probabilities or ranges, duplicate teams, and invalid team data.
- [x] 1.4 Add unit tests for valid default configuration, invalid parameter classes, serialization round trips, and rejection before RNG state is consumed.

## 2. Deterministic Randomness and Matchup Model

- [x] 2.1 Implement stable labeled seed derivation and isolated RNG substreams using a documented platform-independent mixing algorithm.
- [x] 2.2 Implement bounded offense-versus-defense, overall consistency, special-teams, and venue modifiers from current team ratings.
- [x] 2.3 Add paired-seed unit tests for replay determinism, substream isolation, neutral-site symmetry, home advantage activation, and directional rating responses.

## 3. Possession Engine

- [x] 3.1 Define possession context, terminal outcomes, per-possession statistics, and private mutable regulation state with invariant-checking helpers.
- [x] 3.2 Implement deterministic possession sampling for plays, duration, yards, field-position transitions, punts, turnovers, turnover on downs, touchdowns, and field-goal attempts.
- [x] 3.3 Implement integer clock and quarter/halftime boundary handling, kickoff and post-score transitions, and regulation termination after four quarters.
- [x] 3.4 Accumulate points and all required integer box-score fields from possession events, including safe zero-attempt derived-rate helpers.
- [x] 3.5 Add focused tests for each possession outcome, period boundaries, field-position bounds, scoring attribution, possession alternation, and zero-denominator statistics.

## 4. Game Orchestration and Overtime

- [x] 4.1 Implement the pure top-level simulation API that validates inputs, runs regulation, and returns a structured result without exposing partial mutable state.
- [x] 4.2 Implement alternating configurable college-style overtime rounds using an isolated RNG stream until an untied result is reached.
- [x] 4.3 Build the completed existing `Game` representation from simulated period scores while preserving valid lifecycle transitions and recording overtime scoring.
- [x] 4.4 Add end-to-end seeded fixtures and invariant/property-style tests proving score reconciliation, exactly one winner, complete lifecycle, valid ordered possessions, and identical replay.

## 5. Calibration Harness

- [x] 5.1 Define canonical matchup matrices and fixed paired seed sets for equal teams, neutral-site controls, and balanced rating-difference bands.
- [x] 5.2 Implement deterministic sequential batch execution and aggregation for scoring, possessions, turnovers, overtime, home win rate, favorite win rate by band, and upset frequency.
- [x] 5.3 Implement machine-readable calibration reports containing provenance, sample sizes, observed values, expected inclusive bounds, and pass/fail diagnostics.
- [x] 5.4 Establish and document provisional default acceptance envelopes, tune the default profile until the canonical batch passes, and check in the resulting baseline report or snapshot.
- [x] 5.5 Add a fast smoke batch to routine tests and a separately invoked larger canonical suite with deterministic pass/fail behavior.
- [x] 5.6 Benchmark the canonical suite and document its sample size, runtime, invocation, interpretation, and the process for updating profile versions and baselines.

## 6. CLI Integration

- [x] 6.1 Add a CLI matchup-simulation command accepting an explicit seed, neutral/home context, and both teams' current ratings with validated bounds.
- [x] 6.2 Format final score, period scoring, required team statistics, and simulation/profile provenance from the core result without recalculating simulation values.
- [x] 6.3 Add a CLI calibration command or test entry point that emits the aggregate report and returns a failing exit status when an envelope is violated.
- [x] 6.4 Add CLI parsing, reproducible-output, neutral-site, invalid-input, and calibration-status tests.

## 7. Verification and Documentation

- [x] 7.1 Document the simulation command, calibration command, deterministic replay contract, algorithm/profile versioning, initial abstraction limits, and provisional nature of statistical envelopes.
- [x] 7.2 Run rustfmt, Clippy with warnings denied, the complete workspace test suite, and the canonical calibration suite; resolve all failures without weakening acceptance envelopes merely to hide regressions.
- [x] 7.3 Review public APIs for future roster, scheme, coaching, fatigue, weather, and injury modifiers, documenting extension seams without implementing those features.
