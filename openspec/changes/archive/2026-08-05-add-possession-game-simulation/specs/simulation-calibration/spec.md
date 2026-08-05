## ADDED Requirements

### Requirement: Versioned calibration profile
All tunable simulation coefficients, probability bounds, distribution parameters, and statistical acceptance envelopes SHALL be owned by an explicit serializable calibration profile with a stable version identifier. The default profile SHALL be checked into source control and validated before use.

#### Scenario: Result identifies its model
- **WHEN** a game or batch is simulated
- **THEN** its result identifies the simulation algorithm version and calibration profile version used

#### Scenario: Invalid profile is rejected
- **WHEN** a profile violates a declared parameter invariant
- **THEN** validation returns a descriptive error before any games are simulated

### Requirement: Reproducible batch simulation
The calibration harness SHALL simulate a declared matchup matrix over a declared canonical seed set and SHALL produce identical aggregate counts and summary measurements across repeated runs with the same algorithm and profile versions.

#### Scenario: Canonical batch is repeated
- **WHEN** the canonical batch is run twice from the same revision
- **THEN** all raw aggregate counts and derived measurements are identical

#### Scenario: Batch report contains provenance
- **WHEN** a batch completes
- **THEN** its report contains profile version, algorithm version, seed-set identity, matchup definitions, sample size, and every evaluated metric

### Requirement: Aggregate statistical envelopes
The default calibration profile SHALL define inclusive acceptance envelopes for at least points per team, possessions per team, turnovers per team, overtime frequency, equal-team home win rate, favorite win rate by rating-difference band, and upset frequency. The canonical batch SHALL fail validation when any measured value lies outside its envelope.

#### Scenario: Baseline lies inside every envelope
- **WHEN** the canonical batch runs using the checked-in default profile
- **THEN** every required aggregate metric passes its declared acceptance envelope

#### Scenario: Out-of-envelope result is diagnostic
- **WHEN** a measured aggregate falls outside its envelope
- **THEN** the report identifies the metric, observed value, expected bounds, and sample size and returns a failing validation status

### Requirement: Rating response validation
The calibration harness SHALL validate directional and bounded responses to offense, defense, special-teams, and overall rating differences using paired seeds and otherwise identical matchup inputs.

#### Scenario: Stronger composite team wins more often
- **WHEN** a stronger team and weaker team play both venue orientations across the canonical paired seeds
- **THEN** the stronger team wins more often and its win rate remains within the profile's rating-band envelope

#### Scenario: Venue is not confounded with rating
- **WHEN** rating-response matchups are evaluated
- **THEN** each rating pairing is balanced across home and away designations or is played at a neutral site

### Requirement: Statistical tests remain deterministic and practical
Aggregate validation SHALL use fixed finite seed sets and deterministic acceptance envelopes rather than nondeterministic assertions. The routine default test suite SHALL use a sample size that is practical for local development, while a larger canonical calibration suite SHALL be separately runnable for release validation.

#### Scenario: Routine tests run without flaky sampling
- **WHEN** the standard workspace test suite runs repeatedly from the same revision
- **THEN** aggregate assertions have identical outcomes on every run

#### Scenario: Release calibration runs explicitly
- **WHEN** the larger calibration command or ignored test is requested
- **THEN** it evaluates the documented canonical sample size and returns a machine-detectable pass or failure
