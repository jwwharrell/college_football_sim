## Why

The simulator currently represents programs only through aggregate team ratings, so it cannot preserve the players who make up a program across seasons. A validated player and roster domain model is the next foundation for depth charts, development, recruiting, offseason movement, and eventual roster-derived simulation inputs.

## What Changes

- Add stable player identity, biographical data, position, class year, eligibility, redshirt state, and bounded football attributes to the pure simulation domain.
- Add season-scoped program rosters that own players, enforce unique membership, and provide deterministic lookup and ordering.
- Define explicit eligibility progression and roster transition operations that preserve player identity across seasons.
- Add validated serialization contracts and comprehensive domain tests for valid construction, invalid data, duplicate identities, eligibility exhaustion, and deterministic round trips.
- Keep recruiting, transfer acquisition, depth-chart assignment, injuries/fatigue, and player-level game simulation out of scope.

## Capabilities

### New Capabilities

- `player-roster-model`: Pure, validated multi-season player identity, attributes, eligibility, and program roster behavior.

### Modified Capabilities

None. Possession simulation continues to consume aggregate team ratings; roster-derived ratings are deferred to a later change.

## Impact

- Adds player and roster modules and public APIs to `sim_core`.
- Extends domain serialization with version-stable enum and field representations suitable for future persistence adapters.
- Adds unit and serialization tests without introducing I/O, database access, randomness, or new external dependencies.
- Establishes the domain foundation required by roadmap issue #7 and downstream depth-chart, player-state, recruiting, and offseason capabilities.
