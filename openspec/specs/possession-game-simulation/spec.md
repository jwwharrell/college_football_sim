# Possession Game Simulation

## Purpose

Define a deterministic, possession-level college football simulation that uses current team ratings and venue context to produce auditable completed games and reconciled team statistics.

## Requirements

### Requirement: Deterministic simulation contract
The simulation core SHALL accept a matchup, simulation configuration, and explicit `u64` seed and SHALL return the same complete result for identical serialized inputs and simulation version. The simulation API SHALL perform no I/O and SHALL not depend on wall-clock time, global mutable state, thread scheduling, or persistence.

#### Scenario: Identical inputs reproduce a game
- **WHEN** the same matchup and configuration are simulated twice with the same seed and simulation version
- **THEN** the ordered possessions, scoring by period, statistics, and final result are identical

#### Scenario: A different seed can produce a different game
- **WHEN** the same matchup is simulated over a representative set of distinct seeds
- **THEN** the result set contains more than one possession sequence or final score

### Requirement: Possession-level game progression
The engine SHALL simulate an ordered sequence of possessions through regulation. Each possession SHALL identify the possessing team, starting context, period, elapsed or consumed game time, outcome, points, plays, yards, and any turnover, and possession totals SHALL reconcile with the game summary.

#### Scenario: Regulation game produces auditable possessions
- **WHEN** a rated matchup is simulated through four quarters
- **THEN** every scoring event and turnover in the summary is attributable to exactly one ordered possession

#### Scenario: Possession changes are valid
- **WHEN** a possession ends in a punt, score, turnover, missed field goal, or turnover on downs
- **THEN** the next regulation possession and field-position context follow the configured transition rules

### Requirement: Current ratings influence matchup outcomes
The engine SHALL use the current `Team` offense, defense, special-teams, and overall ratings in its probability calculations. Increasing one team's relevant rating while holding all other inputs constant SHALL not reduce that team's expected aggregate performance for the affected dimension across the canonical seed set.

#### Scenario: Offensive strength improves expected production
- **WHEN** otherwise identical teams are compared over the canonical seed set and one team's offense rating is increased
- **THEN** that team's mean points or mean offensive efficiency does not decrease beyond the configured statistical tolerance

#### Scenario: Defensive strength suppresses opponents
- **WHEN** otherwise identical teams are compared over the canonical seed set and one team's defense rating is increased
- **THEN** its opponents' mean points or mean offensive efficiency does not increase beyond the configured statistical tolerance

#### Scenario: Special teams affects relevant events
- **WHEN** otherwise identical teams with different special-teams ratings are simulated over the canonical seed set
- **THEN** the stronger special-teams unit has no worse expected field-goal and field-position contribution beyond the configured statistical tolerance

### Requirement: Venue context affects games
The engine SHALL apply a configurable home-field advantage when a game is not at a neutral site and SHALL apply no home-field modifier at a neutral site.

#### Scenario: Home advantage is enabled
- **WHEN** equal-rated teams play at the designated home team's non-neutral venue over the canonical seed set
- **THEN** the home team wins more than half of completed games and its advantage remains inside the configured calibration envelope

#### Scenario: Neutral site removes home advantage
- **WHEN** equal-rated teams play at a neutral site with home and away designations swapped over paired seeds
- **THEN** neither designation receives the configured home-field modifier

### Requirement: Completed simulations have a winner
The engine SHALL play four regulation quarters and SHALL enter deterministic overtime when regulation ends tied. A successfully simulated game SHALL finish in `Completed` status with exactly one winner and one loser.

#### Scenario: Regulation tie enters overtime
- **WHEN** the teams have equal scores after the fourth quarter
- **THEN** the engine adds one or more ordered overtime periods until the score is no longer tied

#### Scenario: Completed game is internally consistent
- **WHEN** simulation succeeds
- **THEN** period scoring sums to each final score, the winner has the higher score, and the game lifecycle is completed

### Requirement: Structured statistical summary
The result SHALL include team-level points, possessions, plays, total yards, passing yards, rushing yards, turnovers, first downs, third-down attempts and conversions, field-goal attempts and makes, punts, and possession time. Derived rates SHALL be computed from integer counts and SHALL handle zero denominators without invalid numeric values.

#### Scenario: Summary reconciles with possession detail
- **WHEN** a game result is produced
- **THEN** both teams' summary totals equal the corresponding totals accumulated from their possessions

#### Scenario: Derived rate has no attempts
- **WHEN** a statistic has zero attempts
- **THEN** its derived rate is represented by an explicit zero-or-absent convention rather than NaN or infinity

### Requirement: Invalid simulation input is rejected
The simulation API SHALL return a domain error for invalid configuration, identical home and away team identifiers, invalid team data, or unsafe numeric parameters rather than panicking or silently repairing the input.

#### Scenario: Team plays itself
- **WHEN** a matchup supplies the same team identifier for home and away
- **THEN** simulation returns an invalid-parameter error before consuming simulation state

#### Scenario: Invalid probability configuration
- **WHEN** configuration contains a negative weight, non-finite value, impossible probability, or invalid bound ordering
- **THEN** configuration validation reports the offending field and simulation does not begin

### Requirement: CLI matchup simulation
The CLI SHALL provide a command that accepts a seed, venue context, and ratings for two teams, invokes the core simulation API, and prints the final score plus the structured team statistics. CLI presentation SHALL not alter simulation results.

#### Scenario: Seeded CLI game is reproducible
- **WHEN** the same simulation command is run twice with identical arguments
- **THEN** both invocations print equivalent scores and statistics
