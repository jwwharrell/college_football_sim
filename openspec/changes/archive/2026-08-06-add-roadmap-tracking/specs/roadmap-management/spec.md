## ADDED Requirements

### Requirement: Structured roadmap is authoritative
The repository SHALL contain a versioned `roadmap.yaml` whose declared schema defines the authoritative roadmap item identifiers, titles, themes, sequence, statuses, outcomes, exclusions, dependencies, deferral reasons, and delivery evidence. Fields represented in the roadmap SHALL have exactly one manually maintained source of truth.

#### Scenario: Roadmap data is loaded
- **WHEN** the roadmap validation command reads a conforming `roadmap.yaml`
- **THEN** it produces a typed roadmap model containing every declared item without consulting `ROADMAP.md`

#### Scenario: Unknown schema version is supplied
- **WHEN** `roadmap.yaml` declares a schema version unsupported by the validator
- **THEN** validation fails with the unsupported version and supported versions identified

### Requirement: Roadmap items have stable identity and ordered outcomes
Every roadmap item SHALL have a stable identifier matching `<THEME>-<positive integer>`, a non-empty title, a unique positive sequence number, one declared lifecycle status, a non-empty outcome, and at least one explicit exclusion. Identifiers SHALL remain stable when order, title, or status changes.

#### Scenario: Duplicate identity or sequence is present
- **WHEN** two items use the same identifier or sequence number
- **THEN** validation fails and identifies both conflicting items

#### Scenario: Identifier format is invalid
- **WHEN** an item identifier does not use an uppercase theme and positive numeric suffix separated by a hyphen
- **THEN** validation fails and identifies the invalid identifier

### Requirement: Dependencies are valid and acyclic
Each dependency SHALL reference another roadmap item, SHALL not reference the containing item, and the complete dependency graph SHALL be acyclic. An item with a status of `proposed`, `active`, or `complete` SHALL not depend on an item whose status is `exploring` or `deferred`.

#### Scenario: Dependency is missing
- **WHEN** an item names an identifier absent from the roadmap
- **THEN** validation fails with the item and missing dependency

#### Scenario: Dependency cycle exists
- **WHEN** roadmap dependencies form a direct or transitive cycle
- **THEN** validation fails and reports a cycle path

#### Scenario: Delivery outruns a dependency
- **WHEN** a proposed, active, or complete item depends on an exploring or deferred item
- **THEN** validation fails and reports the incompatible statuses

### Requirement: Lifecycle states require repository evidence
The roadmap SHALL support `exploring`, `proposed`, `active`, `complete`, and `deferred`. A proposed or active item SHALL reference an existing active OpenSpec change. A complete item SHALL reference at least one existing main capability spec or archived OpenSpec change. A deferred item SHALL include a non-empty reason. Exploring items SHALL not be required to carry implementation evidence.

#### Scenario: Active item references no active change
- **WHEN** an item is proposed or active without naming a change directory under `openspec/changes/`
- **THEN** validation fails and identifies the missing evidence

#### Scenario: Completed item has durable evidence
- **WHEN** a complete item references an existing capability under `openspec/specs/` or a change under `openspec/changes/archive/`
- **THEN** its lifecycle evidence validation passes

#### Scenario: Deferred item lacks rationale
- **WHEN** an item has deferred status and no non-empty deferral reason
- **THEN** validation fails and identifies the item

### Requirement: Lifecycle transitions are documented
The roadmap workflow SHALL define the normal transition `exploring → proposed → active → complete`, SHALL permit any non-complete item to become deferred, and SHALL permit deferred items to return to exploring. Completion SHALL occur only after implementation, main-spec synchronization, and archival provide durable evidence.

#### Scenario: Feature discovery begins
- **WHEN** a future feature is added without an implementation proposal
- **THEN** it is recorded as exploring and does not imply committed scope or schedule

#### Scenario: Delivery is finalized
- **WHEN** an active feature is implemented, its delta specs are synchronized, and its change is archived
- **THEN** the roadmap item can become complete with references to the resulting main spec or archive

### Requirement: Human roadmap is deterministic and drift-free
`ROADMAP.md` SHALL be generated deterministically from `roadmap.yaml` and SHALL present lifecycle definitions, an ordered summary, outcomes, exclusions, dependencies, and evidence links. A check mode SHALL fail if the committed Markdown differs byte-for-byte from freshly rendered output and SHALL not modify files.

#### Scenario: Generated roadmap is current
- **WHEN** render check runs and `ROADMAP.md` matches the canonical rendering
- **THEN** the command succeeds without changing either file

#### Scenario: Generated roadmap is stale
- **WHEN** `roadmap.yaml` changes without regenerating `ROADMAP.md`
- **THEN** render check fails with instructions for regenerating the document

### Requirement: Roadmap commands are safe and diagnostic
The CLI SHALL expose commands to validate the roadmap, render `ROADMAP.md`, and check generated-file consistency. Validation and check commands SHALL perform no writes, rendering SHALL write only the declared generated roadmap target, and all failures SHALL identify the item, field, or reference responsible and return a non-zero exit status.

#### Scenario: Validation succeeds
- **WHEN** all schema, identity, dependency, lifecycle, and evidence rules pass
- **THEN** the command reports the item count and succeeds

#### Scenario: Multiple independent errors exist
- **WHEN** the roadmap contains multiple independently detectable validation failures
- **THEN** the command reports all detected failures in deterministic order before returning failure

### Requirement: Initial dynasty roadmap is captured
The initial roadmap SHALL include statistical game simulation as complete and SHALL include ordered items for player and roster modeling, positions and depth charts, schedules and the weekly season loop, development/fatigue/injuries, recruiting, coaching/schemes/tactics, transfers/graduation/offseason progression, program prestige/facilities/finances/expectations, rankings/bowls/conference championships/playoff, and records/awards/historical continuity.

#### Scenario: Initial roadmap is rendered
- **WHEN** `ROADMAP.md` is generated from the initial structured roadmap
- **THEN** all eleven roadmap outcomes appear in declared sequence with their statuses and dependencies

### Requirement: Verification detects roadmap drift
The repository's standard automated verification SHALL run roadmap semantic validation and generated-document consistency checks alongside existing formatting, linting, and tests.

#### Scenario: Pull request introduces invalid roadmap data
- **WHEN** automated verification runs against a roadmap with invalid evidence or a stale generated document
- **THEN** verification fails before the change can be treated as roadmap-consistent
