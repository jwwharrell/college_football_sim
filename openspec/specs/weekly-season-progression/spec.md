# Weekly Season Progression

## Purpose

Define deterministic, atomic weekly game simulation, standings updates, replay provenance, and regular-season completion for dynasty season state.

## Requirements

### Requirement: Deterministic weekly simulation
The season loop SHALL accept an explicit season seed and validated simulation configuration, derive one stable seed for each game from the season identity and game identifier, and invoke the existing deterministic game simulator for every scheduled game in the current week.

#### Scenario: Identical season inputs reproduce a week
- **WHEN** equivalent pre-week season states are advanced with the same season seed, schedule, teams, and simulation configuration
- **THEN** they produce identical per-game simulation results, records, standings, and post-week season state

#### Scenario: Schedule storage order does not affect results
- **WHEN** equivalent schedules contain the same identified games in different collection orders
- **THEN** every game receives the same derived seed and produces the same result

#### Scenario: Game seeds are isolated
- **WHEN** an unrelated game is added to or removed from another week without changing an existing game's identity
- **THEN** the existing game's derived seed remains unchanged

### Requirement: Weekly advancement is atomic
The season loop SHALL validate and simulate the complete current week before committing any result, standings change, or week counter change. If any game or configuration is invalid or any simulation fails, it SHALL return an error with the original season state unchanged.

#### Scenario: Entire valid week commits
- **WHEN** every scheduled game in the current week simulates successfully
- **THEN** all results are committed, records are refreshed, and the current week advances exactly once

#### Scenario: One game fails
- **WHEN** any current-week game cannot be prepared or simulated
- **THEN** no current-week game result, record update, or week advancement is committed

#### Scenario: Empty week advances
- **WHEN** the current week has no games
- **THEN** the season commits no game results and advances to the next week exactly once

### Requirement: Only the current unplayed week can advance
The system SHALL reject attempts to skip ahead, replay a committed week, advance a season whose current week is partially or already resolved, or advance after the regular season is complete.

#### Scenario: Repeated advancement request cannot replay games
- **WHEN** advancement is requested again from state that contains a completed result for the current week
- **THEN** the system returns a season-state error without resimulating or overwriting a result

#### Scenario: Completed regular season cannot advance
- **WHEN** advancement is requested after the final configured week has committed
- **THEN** the system returns a season-complete error and preserves the completed state

### Requirement: Completed results preserve replay provenance
For every simulated schedule entry, the season state SHALL retain the completed game and sufficient simulation provenance to identify its derived seed, simulation algorithm version, and calibration profile version.

#### Scenario: Completed game can be audited
- **WHEN** a current-week game is committed
- **THEN** its stored result identifies the schedule game, derived seed, algorithm version, profile version, final score, and structured simulation output

### Requirement: Standings reflect committed results
After a week commits, the system SHALL derive overall and conference records from all committed completed games exactly once and SHALL expose standings in a deterministic total order. This deterministic order is presentation stability, not a rankings or postseason selection rule.

#### Scenario: Conference game updates both record dimensions
- **WHEN** a conference game commits with one winner and one loser
- **THEN** the winner gains one overall and conference win and the loser gains one overall and conference loss

#### Scenario: Non-conference game updates overall records only
- **WHEN** a non-conference game commits
- **THEN** both teams' overall records change and their conference records do not

#### Scenario: Tied records have stable presentation order
- **WHEN** two teams have equal standings metrics
- **THEN** repeated standings queries return them in the same documented identifier-based fallback order

### Requirement: Regular-season completion
The season SHALL become complete immediately after the final configured week commits, while preserving its full schedule, completed results, records, and replay provenance.

#### Scenario: Final week completes the season
- **WHEN** the configured final week commits successfully
- **THEN** the season reports complete and exposes no next regular-season week

### Requirement: CLI season-loop harness
The CLI SHALL provide commands that can display a deterministic schedule and simulate either the current week or the full regular season from explicit seed inputs, without placing I/O or presentation behavior in the simulation core.

#### Scenario: Seeded CLI season is reproducible
- **WHEN** the same season-loop command is invoked twice with identical inputs
- **THEN** both invocations display equivalent weekly results, final records, and completion state
