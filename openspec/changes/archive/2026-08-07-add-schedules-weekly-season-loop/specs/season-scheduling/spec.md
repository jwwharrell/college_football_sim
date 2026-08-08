## ADDED Requirements

### Requirement: Validated regular-season schedule
The system SHALL construct a regular-season schedule only when every game has a non-empty stable identifier, references two distinct teams in the season, and is assigned to a week from 1 through the season's configured total weeks.

#### Scenario: Valid schedule is accepted
- **WHEN** every scheduled matchup has a unique identifier, two participating season teams, and an in-range week
- **THEN** the system accepts the schedule and exposes its games in canonical week-and-game-identifier order

#### Scenario: Unknown team is rejected
- **WHEN** a scheduled matchup references a team identifier that is not part of the season
- **THEN** schedule construction returns a domain error identifying the game and unknown team

#### Scenario: Invalid week is rejected
- **WHEN** a scheduled matchup is assigned to week zero or a week after the configured regular season
- **THEN** schedule construction returns a domain error identifying the game and invalid week

### Requirement: Stable game identities are unique
The schedule SHALL reject empty or duplicate game identifiers so every scheduled result and derived simulation seed has one unambiguous identity.

#### Scenario: Duplicate game identifier is rejected
- **WHEN** two schedule entries use the same game identifier
- **THEN** schedule construction fails before a season can advance

### Requirement: A team plays at most once per week
The schedule SHALL reject any week in which a team appears in more than one game, while allowing a team to have no game in a week.

#### Scenario: Weekly conflict is rejected
- **WHEN** a team is assigned as home or away in two games during the same week
- **THEN** schedule construction returns a conflict error that identifies the team, week, and games

#### Scenario: Bye week is accepted
- **WHEN** a participating team has no scheduled game in an otherwise valid week
- **THEN** the schedule remains valid and the team receives no result for that week

### Requirement: Schedule venue and competition metadata
Each scheduled game SHALL preserve its home team, away team, location, neutral-site status, and conference-game status for use by simulation and standings.

#### Scenario: Schedule entry becomes a simulation matchup
- **WHEN** a scheduled game is prepared for simulation
- **THEN** its teams, week, location, venue context, and conference-game flag match the validated schedule entry
