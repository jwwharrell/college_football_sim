# Depth Chart Management

## Purpose

Define validated manager-controlled positional depth charts and deterministic roster-derived unit strengths for optional simulation composition.

## Requirements

### Requirement: Depth charts represent typed positional units
The domain SHALL represent offense, defense, and special-teams depth charts through closed, serializable unit and slot types. Repeated roles SHALL use stable ordinals, each slot SHALL preserve explicit player depth order, and the canonical chart SHALL define every required starter slot for all three units.

#### Scenario: Complete chart is constructed
- **WHEN** a caller supplies every canonical starter slot and ordered assignments from one roster
- **THEN** the domain returns a chart carrying the roster team identity, season year, typed slots, and declared depth order

#### Scenario: Required starter is missing
- **WHEN** any canonical starter slot is absent or has no assigned player
- **THEN** chart construction fails with an error identifying the missing slot

### Requirement: Assignments reference eligible roster members
Every depth-chart assignment SHALL reference a player in the chart's team-and-season roster whose eligibility reports at least one season remaining. Construction and edits SHALL reject missing, wrong-roster, or exhausted players without partially changing the chart.

#### Scenario: Eligible roster member is assigned
- **WHEN** an eligible player from the chart's roster is assigned to a compatible slot
- **THEN** the assignment succeeds and retains the player's stable identifier

#### Scenario: Non-member is assigned
- **WHEN** an assignment references a player identifier absent from the roster
- **THEN** validation fails, identifies the player and slot, and leaves the chart unchanged

#### Scenario: Exhausted player is assigned
- **WHEN** a roster member with zero seasons remaining is assigned
- **THEN** validation fails and leaves the chart unchanged

### Requirement: Slot assignments enforce position compatibility
Each slot SHALL declare compatible primary positions. Exact-position players SHALL be accepted; `Athlete` SHALL be accepted for non-specialist scrimmage slots; and kicker, punter, and long-snapper slots SHALL require their matching specialist position.

#### Scenario: Compatible primary position is assigned
- **WHEN** a player's primary position is allowed by the target slot
- **THEN** the player can occupy any declared depth rank in that slot

#### Scenario: Incompatible position is rejected
- **WHEN** a player's primary position is not allowed by the target slot
- **THEN** assignment fails with an error identifying the player and slot

#### Scenario: Athlete cannot replace specialist
- **WHEN** an athlete is assigned to kicker, punter, or long snapper
- **THEN** assignment fails without changing the chart

### Requirement: Unit assignments are unique and edits are atomic
A player identifier SHALL appear at most once within a unit, although one player MAY appear in different units. Reordering, assigning, and removing players SHALL validate the complete resulting chart and SHALL either commit the entire edit or preserve the prior chart.

#### Scenario: Duplicate within one unit is rejected
- **WHEN** a player already assigned in an offensive slot is assigned to another offensive slot
- **THEN** the edit fails and the original offensive chart is unchanged

#### Scenario: Cross-unit specialist overlap is allowed
- **WHEN** a roster member has compatible assignments in two different units
- **THEN** the complete chart remains valid

#### Scenario: Manager reorders a slot
- **WHEN** a manager supplies a valid new ordered list for an existing slot
- **THEN** the chart records that exact depth order as one atomic edit

### Requirement: Chart access and serialization are deterministic
Canonical iteration and serialization SHALL order units and slots by their typed canonical order while preserving player depth order within each slot. Valid charts SHALL support lossless serde round trips using stable snake-case enum representations and SHALL perform no I/O.

#### Scenario: Construction order differs
- **WHEN** equivalent slot assignments are supplied in different collection insertion orders
- **THEN** canonical iteration and serialized chart output are identical

#### Scenario: Chart round trip succeeds
- **WHEN** a valid complete chart is serialized and deserialized
- **THEN** team identity, season, slots, depth order, and canonical output are unchanged

### Requirement: Unit strengths are bounded and reproducible
The domain SHALL derive offense, defense, and special-teams strength values from assigned players' foundational attributes using fixed documented slot, attribute, and depth weights. Strengths SHALL use deterministic arithmetic, SHALL be integers from 0 through 100, and SHALL not use randomness or mutable external state.

#### Scenario: Identical chart reproduces strengths
- **WHEN** unit strengths are calculated repeatedly from the same roster and chart
- **THEN** all three strength values are identical and within 0 through 100

#### Scenario: Stronger starter improves relevant unit
- **WHEN** one compatible starter is replaced by a player with higher values for every attribute weighted by that slot and all other assignments remain equal
- **THEN** the relevant unit strength does not decrease

#### Scenario: Backup cannot outweigh starter
- **WHEN** the same two compatible players exchange starter and backup ranks
- **THEN** placing the stronger player first produces a unit strength at least as high as placing that player second

### Requirement: Depth chart scope excludes live personnel state
Depth charts SHALL describe manager-selected pregame ordering only and SHALL NOT model in-game substitutions, tactical packages, injuries, fatigue, or snap-by-snap availability.

#### Scenario: Chart is evaluated for simulation input
- **WHEN** strengths are derived from a valid chart
- **THEN** the result depends only on roster data, assignments, and versioned weights, with no live-game state
