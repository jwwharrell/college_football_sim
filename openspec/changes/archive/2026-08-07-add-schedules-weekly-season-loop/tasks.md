## 1. Schedule Domain

- [x] 1.1 Add serializable schedule-entry and schedule types with canonical `(week, game_id)` ordering and week/team lookup APIs.
- [x] 1.2 Implement validated season construction for unique team and game IDs, known distinct opponents, valid week bounds, and one game per team per week while allowing byes.
- [x] 1.3 Add focused schedule tests for valid metadata preservation, canonical ordering, empty/duplicate IDs, unknown/self opponents, invalid weeks, conflicts, and byes.

## 2. Season State and Deterministic Seeds

- [x] 2.1 Refactor `Season` to own the validated schedule and completed `SimulationResult` values keyed by stable game ID, with serializable replay provenance preserved.
- [x] 2.2 Add versioned, length-delimited per-game seed derivation from season seed, year, and game ID using the existing stable derivation primitive.
- [x] 2.3 Add tests proving game seeds and results are stable across repeated runs and schedule input order, isolated from unrelated games, and distinct for distinct identities.

## 3. Atomic Weekly Progression

- [x] 3.1 Implement matchup construction from schedule entries and season teams, preserving venue, location, week, conference flag, and default modifier behavior.
- [x] 3.2 Implement all-or-nothing current-week advancement that validates configuration/state, buffers every simulation result, commits the full week, and advances exactly once, including empty weeks.
- [x] 3.3 Add explicit domain errors for invalid schedule/progression states, repeated or partial weeks, and advancement after regular-season completion.
- [x] 3.4 Add tests showing a failing game/configuration leaves results, records, and week unchanged and that completed weeks cannot be replayed or overwritten.
- [x] 3.5 Add full-season progression through the final week and verify completion preserves schedule, results, and provenance.

## 4. Records and Standings

- [x] 4.1 Rebuild overall and conference records idempotently from committed results after each successful week.
- [x] 4.2 Implement deterministic standings comparison with documented metrics and ascending team ID as the final presentation-only fallback.
- [x] 4.3 Add tests for conference/non-conference record updates, no double counting, stable tied ordering, byes, and multi-week standings.

## 5. CLI Integration

- [x] 5.1 Add CLI commands/options to display a deterministic schedule and simulate one week or the full regular season from an explicit seed.
- [x] 5.2 Format weekly scores, standings, provenance, and completion state without moving I/O into `sim_core`.
- [x] 5.3 Add CLI integration tests proving identical inputs produce equivalent schedule, weekly, and full-season output and invalid inputs return non-zero errors.

## 6. Compatibility and Verification

- [x] 6.1 Migrate existing season call sites and tests from permissive game/week mutation to the validated schedule and progression APIs.
- [x] 6.2 Update README documentation with the season schedule/loop contract, deterministic seed behavior, exclusions, and CLI examples.
- [x] 6.3 Run `cargo fmt --check`, `cargo test --workspace`, and workspace Clippy with warnings denied; fix all failures.
- [x] 6.4 Validate the OpenSpec change and confirm every season-scheduling and weekly-season-progression scenario has corresponding automated coverage.
