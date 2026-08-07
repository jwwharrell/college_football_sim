## Why

The simulator now preserves players and season rosters, but managers cannot organize those players into playable units or express who starts and who backs them up. GitHub issue #15 (`DYNASTY-02`) adds the missing bridge from roster construction to deterministic, player-informed team strengths used by simulation.

## What Changes

- Add validated offensive, defensive, and special-teams depth charts with named slots and ordered player assignments.
- Enforce roster membership, player eligibility, position compatibility, unique assignments within a unit, required starters, and deterministic ordering at construction and update boundaries.
- Derive bounded, deterministic offense, defense, and special-teams strength values from assigned players' foundational attributes and depth order.
- Expose the derived unit strengths through the existing simulation modifier seam while preserving seeded determinism and current aggregate `Team` ratings as baseline inputs.
- Add serializable domain APIs and tests for valid charts, invalid assignments, atomic edits, deterministic round trips, strength calculation, and simulation effects.
- Keep in-game substitutions, tactical personnel packages, injuries, and fatigue out of scope.

## Capabilities

### New Capabilities
- `depth-chart-management`: Validated positional units, ordered depth charts, manager assignment operations, and deterministic roster-derived unit strengths.

### Modified Capabilities
- `possession-game-simulation`: Allow explicitly supplied roster-derived unit strengths to populate the existing matchup modifier seam and influence simulations deterministically.

## Impact

- Adds depth-chart and unit-strength domain APIs to `sim_core`, building on `player` and `roster` without adding I/O or randomness.
- Extends simulation input composition while retaining aggregate team ratings and the existing seeded possession engine.
- Adds serde contracts, unit tests, integration/regression tests, calibration verification, and current project documentation.
- Introduces no new external dependencies and no persistence or CLI management workflow in this change.
