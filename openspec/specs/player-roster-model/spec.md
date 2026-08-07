# Player Roster Model Specification

## Purpose

Define pure, validated, deterministic player identity, eligibility, and season-scoped roster behavior without coupling rosters to current possession-simulation inputs.

## Requirements

### Requirement: Players have stable validated identity
The domain SHALL represent each player with a stable identifier, non-empty first and last names, and a primary football position. A player's identifier SHALL remain unchanged across roster and season transitions, and invalid identity or name data SHALL return a domain error rather than panic or be silently normalized.

#### Scenario: Valid player is constructed
- **WHEN** a caller supplies a unique non-empty identifier, non-empty names, a supported position, valid attributes, and coherent eligibility
- **THEN** the domain returns a player carrying those values

#### Scenario: Player identity is invalid
- **WHEN** a player identifier or required name is empty or whitespace-only
- **THEN** construction fails with an invalid-parameter error identifying the field

### Requirement: Positions and foundational attributes are typed and bounded
The domain SHALL provide a closed, serializable set of common roster positions plus an athlete fallback. Each player SHALL have named foundational speed, strength, agility, awareness, and stamina ratings represented as integers from 0 through 100 inclusive, and SHALL reject any rating outside that range.

#### Scenario: Boundary attribute ratings are valid
- **WHEN** a player has any foundational rating equal to 0 or 100
- **THEN** attribute validation succeeds

#### Scenario: Attribute rating exceeds the bound
- **WHEN** a foundational rating is greater than 100
- **THEN** validation fails and identifies the offending attribute

### Requirement: Eligibility state is explicit and coherent
Each player SHALL carry an explicit class year, seasons played, seasons remaining, and redshirt-used state. Seasons played and seasons remaining SHALL each be within 0 through 4 and their sum SHALL not exceed 4. A player with zero seasons remaining SHALL be ineligible to return for another season.

#### Scenario: Returning player has eligibility
- **WHEN** a player's eligibility reports at least one season remaining
- **THEN** the player is reported as eligible to return

#### Scenario: Eligibility totals are incoherent
- **WHEN** seasons played plus seasons remaining exceeds four
- **THEN** player validation fails with an eligibility error

#### Scenario: Eligibility is exhausted
- **WHEN** a player has zero seasons remaining
- **THEN** the player is reported as unable to return without changing any other player data

### Requirement: Season participation advances eligibility deterministically
The domain SHALL advance a player's class year and eligibility from an explicit season participation outcome. Consuming a season SHALL increment seasons played and decrement seasons remaining; a permitted redshirt season SHALL mark the redshirt as used without consuming a season. Advancing an exhausted player or applying a second redshirt SHALL return a domain error.

#### Scenario: Player consumes a season
- **WHEN** an eligible player advances with a season-used outcome
- **THEN** class year advances, seasons played increases by one, seasons remaining decreases by one, and player identity is unchanged

#### Scenario: Player uses an available redshirt
- **WHEN** an eligible player who has not redshirted advances with a redshirt outcome
- **THEN** class year advances, redshirt-used becomes true, seasons played and seasons remaining are unchanged, and player identity is unchanged

#### Scenario: Player attempts a second redshirt
- **WHEN** a player whose redshirt-used state is true advances with a redshirt outcome
- **THEN** advancement fails without partially changing the player

### Requirement: Rosters enforce season-scoped unique membership
A roster SHALL identify one non-empty program team identifier and one positive season year and SHALL contain only valid players with unique player identifiers. Construction and membership mutation SHALL reject duplicate identities and invalid members without partially modifying the roster.

#### Scenario: Program roster is constructed
- **WHEN** a caller supplies a team identifier, season year, and valid players with unique identifiers
- **THEN** the roster is created and every player can be retrieved by stable identifier

#### Scenario: Duplicate player is supplied
- **WHEN** two roster entries share a player identifier or an existing identifier is added again
- **THEN** the operation fails, identifies the duplicate, and leaves roster membership unchanged

#### Scenario: Player is removed
- **WHEN** an existing player identifier is removed from a roster
- **THEN** the returned player retains all profile and eligibility data and is no longer a roster member

### Requirement: Roster access and serialization are deterministic
Roster iteration and canonical serialization SHALL order players by stable player identifier independent of insertion order. Player, attribute, eligibility, position, and roster values SHALL support lossless serde round trips using stable snake-case enum representations and SHALL perform no I/O.

#### Scenario: Insertion order differs
- **WHEN** equivalent players are inserted into two rosters in different orders
- **THEN** ordered iteration and canonical serialized player order are identical

#### Scenario: Roster round trip succeeds
- **WHEN** a valid roster is serialized and deserialized
- **THEN** the resulting roster is equal in identity, season, membership, player data, and deterministic ordering

### Requirement: Next-season transition preserves returning identity
The domain SHALL create a next-season roster from a valid roster and an explicit participation outcome for every member. Eligible players SHALL retain stable identity and profile data while advancing eligibility, players whose eligibility becomes exhausted SHALL be reported as departures and omitted from the next roster, and any invalid or missing outcome SHALL fail the entire transition without mutating the source roster.

#### Scenario: Eligible roster advances
- **WHEN** every roster member has a valid participation outcome and remains eligible after advancement
- **THEN** the next roster has the same team identifier, the following season year, the same player identifiers, and advanced eligibility

#### Scenario: Player exhausts eligibility
- **WHEN** a player's final remaining season is consumed during transition
- **THEN** the transition summary lists that player as an eligibility departure and the next roster omits the player

#### Scenario: Participation outcome is missing
- **WHEN** any roster member has no declared participation outcome
- **THEN** transition fails and neither the source roster nor any player is modified

### Requirement: Player rosters do not alter game simulation yet
The player and roster model SHALL remain independent of possession simulation inputs and SHALL not derive or replace current aggregate team ratings in this capability.

#### Scenario: Roster is created beside a rated team
- **WHEN** a program has both a valid roster and an aggregate-rated `Team`
- **THEN** existing seeded possession simulations produce the same results as before this capability unless a future integration explicitly supplies roster-derived modifiers
