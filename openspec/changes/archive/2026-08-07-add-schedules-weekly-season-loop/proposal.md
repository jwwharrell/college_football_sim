## Why

The simulator can resolve an individual matchup, but a dynasty cannot yet define a valid season schedule or advance that schedule week by week. SEASON-01 adds the orchestration needed to turn deterministic game simulation into reproducible season state while keeping rankings, postseason selection, and schedule optimization out of scope.

## What Changes

- Add validated season schedules with stable game identities, week assignments, venue and conference metadata, and safeguards against duplicate or conflicting matchups.
- Add a deterministic weekly season loop that simulates every scheduled game in a week, records completed results, refreshes standings, and advances the season exactly once.
- Derive per-game seeds from dynasty/season seed plus stable game identity so results do not depend on collection iteration order and can be replayed.
- Make weekly advancement an all-or-nothing domain transition that rejects incomplete, repeated, out-of-order, or post-season advances without leaving partial state.
- Expose CLI commands to inspect a schedule and exercise deterministic week or full-regular-season advancement.

## Capabilities

### New Capabilities

- `season-scheduling`: Defines valid, deterministic regular-season schedules and their matchup/week constraints.
- `weekly-season-progression`: Defines atomic weekly simulation, standings updates, seed derivation, replay guarantees, and regular-season completion.

### Modified Capabilities

None.

## Impact

- Extends `sim_core` season-domain APIs and integrates them with the existing pure possession simulation contract.
- Adds explicit schedule/progression errors and serializable season state needed by future persistence work.
- Extends the CLI with a season-loop harness; no database migration or new external dependency is required.
- Establishes the season-state boundary future dynasty persistence will save, without implementing persistence, rankings, postseason selection, or conference schedule optimization.
