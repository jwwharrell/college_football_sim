## Context

`sim_core` currently models a `Team` through aggregate ratings and stores teams directly in games and seasons. There is no player identity or roster ownership, so later depth-chart, development, recruiting, and offseason systems have no stable domain foundation. The new model must remain pure, serializable, deterministic, and independent of persistence or user-interface concerns.

## Goals / Non-Goals

**Goals:**

- Represent players with stable identity, names, primary football position, bounded foundational attributes, and explicit eligibility state.
- Represent a program's roster for a specific season with validated, unique membership and deterministic access.
- Advance returning players between seasons without changing identity and remove players whose eligibility is exhausted.
- Provide serde-compatible domain values and focused APIs that future persistence, depth-chart, and player-state systems can compose.

**Non-Goals:**

- Recruiting, transfer portal, walk-on, or graduation-selection workflows.
- Depth-chart slots, position-group eligibility rules, or in-game personnel packages.
- Development, fatigue, injuries, morale, academics, or discipline.
- Deriving aggregate team ratings or changing possession simulation behavior.
- Reproducing every historical NCAA eligibility exception.

## Decisions

### Keep players and rosters in dedicated `sim_core` modules

`player.rs` will own player identity, position, attributes, and eligibility; `roster.rs` will own season-scoped program membership and transitions. Keeping them separate prevents the existing `Team` aggregate from becoming a large mutable object and lets future persistence store teams, players, and rosters independently. Embedding `Vec<Player>` directly in `Team` was rejected because games currently clone teams as compact matchup inputs.

### Use explicit value types instead of free-form strings

`PlayerId` and `TeamId` remain serialized string identifiers but are validated as non-empty trimmed values. `Position` and `ClassYear` use enums with stable snake-case serde names. `PlayerAttributes` uses named `u8` ratings bounded to 0–100 rather than a string-keyed map, making invalid and unknown attributes impossible to silently accept.

The initial position set will cover common football roster identities: quarterback, running back, fullback, wide receiver, tight end, offensive line, defensive line, edge, linebacker, cornerback, safety, kicker, punter, long snapper, and athlete. Fine-grained line positions and depth-chart eligibility remain extensible follow-up work.

### Model eligibility as state, not inferred age

Eligibility will explicitly record `class_year`, `seasons_played`, `seasons_remaining`, and whether a redshirt has been used. Construction enforces coherent bounds: played and remaining seasons cannot exceed four in total, and an exhausted player cannot be advanced as returning. A season transition accepts an explicit participation outcome (`SeasonUsed` or `Redshirted`) rather than inferring it from games, because player-level participation is outside this change.

This intentionally simplified model is version-neutral domain state. Historical waiver and medical-redshirt policies can later change transition policy without changing player identity.

### Make roster invariants authoritative at construction and mutation boundaries

A `Roster` contains a non-empty team ID, a positive season year, and players keyed conceptually by stable player ID. Public construction and membership operations reject duplicate IDs and invalid players. Read APIs return deterministic player-ID ordering regardless of insertion order, supporting reproducible tests and serialization.

Generic add/remove operations express membership maintenance only; they do not decide why a player joins or leaves. Recruiting and transfer systems will call these boundaries later after applying their own rules.

### Produce next-season rosters as new values

Transitioning a roster returns a new roster for the next year. Returning players retain IDs and immutable profile data, their eligibility advances according to explicit participation outcomes, and exhausted players are omitted with a transition summary. This avoids partially mutated rosters when one player has invalid transition data and fits the repository's preference for deterministic, testable domain operations.

## Risks / Trade-offs

- **Simplified eligibility diverges from some NCAA exceptions** → Keep policy explicit and isolated so waivers can be added without rewriting roster ownership.
- **A broad position enum may be insufficient for depth charts** → Include an `Athlete` fallback and extend positions under the dedicated depth-chart change.
- **Named generic attributes do not capture position skills** → Limit this change to foundational attributes; add position-specific skills with the simulation integration that consumes them.
- **Deterministic ordering can add sorting cost** → Rosters are small enough that clarity and stable output outweigh premature indexing optimization.
- **Generic membership operations could bypass future acquisition rules** → Treat them as low-level domain boundaries; higher-level recruiting and transfer services own acquisition policy.

## Migration Plan

1. Add player and roster domain modules with public value types and validation.
2. Add constructors, lookup/membership APIs, and next-season transition behavior.
3. Add unit tests, serde round-trip tests, and deterministic ordering tests.
4. Export the modules from `sim_core` and document the new boundary without changing existing `Team`, `Season`, or simulation inputs.

No stored data migration is required because no player persistence exists. Rollback removes the new modules and exports without changing current game or season behavior.

## Open Questions

- Should the later depth-chart capability split offensive line and defensive line into exact positions, or model exact alignment as a depth-chart role separate from a player's primary position?
