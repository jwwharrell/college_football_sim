## Context

`sim_core` now has stable players, broad primary positions, bounded foundational attributes, and deterministic season rosters. Possession simulation still uses aggregate `Team` ratings plus a public `MatchupModifiers` seam. Issue #15 needs a pure domain layer that lets managers order rostered players at playable positions and turns those choices into deterministic unit strengths without introducing player-level game state.

## Goals / Non-Goals

**Goals:**

- Represent offense, defense, and special-teams slots and ordered depth assignments with stable serde names.
- Validate complete starting lineups against one season roster, player eligibility, position compatibility, and assignment uniqueness.
- Support atomic manager edits and canonical iteration/serialization.
- Calculate explainable 0–100 unit strengths from foundational player attributes and depth order.
- Convert strengths into bounded simulation modifiers through an explicit adapter, preserving seeded determinism.

**Non-Goals:**

- In-game substitution logic, snap counts, tactical formations, or personnel packages.
- Injury, fatigue, morale, development, or position-training effects.
- Recruiting, transfer, roster acquisition, persistence, or UI workflows.
- Replacing aggregate `Team` ratings or simulating individual player statistics.

## Decisions

### Model a canonical chart with typed unit and slot keys

A new `depth_chart` module will define `Unit` (`Offense`, `Defense`, `SpecialTeams`), typed `DepthChartSlot` values, and a canonical required-starter template. Repeated roles carry a stable ordinal (for example wide receiver 1–3 and offensive line 1–5), while each slot owns an ordered, non-empty list of `PlayerId` values. Typed slots were chosen over free-form labels so unknown roles, duplicate slot keys, and unstable names cannot enter persisted domain state. Multiple formation-specific templates are deferred because they would imply tactical packages.

### Use explicit, conservative position compatibility

Each slot declares the existing `Position` values it accepts; `Athlete` is compatible with any non-specialist scrimmage slot but not kicker, punter, or long snapper. Exact primary-position matches remain the normal path. Compatibility belongs beside slot definitions rather than in `Player`, allowing future depth-chart roles to become more precise without changing stable player identity.

### Validate charts against a roster at every construction or edit boundary

A chart carries the roster's team ID and season year and is created with a roster reference. Validation requires every referenced player to exist on that roster, have remaining eligibility, fit the slot, occur at most once within a unit, and fill every required starter slot. A manager edit produces a validated new chart (or commits only after full validation), preventing partial mutation. Players may appear on offense or defense and special teams because two-way and specialist overlap is legitimate; duplicate use inside one unit is rejected.

### Keep ordering canonical and serialization data-oriented

Slots serialize in unit/slot order and player IDs serialize in declared depth order. Lookup uses typed keys; canonical collections or constructor sorting remove insertion-order effects. Deserialization is validated through the same invariants rather than accepting malformed charts. This follows the roster model's deterministic boundary while preserving meaningful depth order.

### Derive strengths with fixed integer arithmetic

Position slots use documented fixed weights over speed, strength, agility, awareness, and stamina. A starter contributes full weight and backups contribute decreasing predefined depth weights; deeper entries cannot outweigh the starter. Slot scores combine into offense, defense, and special-teams scores using fixed integer weights, round once, and clamp to 0–100. Integer/rational arithmetic was chosen over floating-point aggregation to make results portable and byte-for-byte reproducible.

### Adapt strengths into the existing modifier seam

An explicit pure adapter converts a chart's 0–100 unit strengths to `MatchupModifiers` in the existing -25 through 25 bounds, using 50 as neutral. Callers opt in when composing a matchup; `simulate_game` continues to consume only its serialized matchup, configuration, and seed. Aggregate `Team` ratings remain baseline inputs, so existing callers and unmodified matchups retain their results.

## Risks / Trade-offs

- **A single canonical lineup cannot express every scheme** → Keep slot types and templates isolated so later tactical-package work can add formations without changing player IDs.
- **Broad primary positions make some assignments coarse** → Use explicit compatibility tables and avoid inventing fine-grained player positions in this change.
- **Foundational attributes produce simplified unit strengths** → Make weights named, documented, and tested; position-specific skills can replace or extend them later.
- **Allowing cross-unit reuse can overstate roster versatility** → Permit it deliberately for two-way players while rejecting duplication within a unit; workload belongs to fatigue/snap systems.
- **Modifier conversion may shift calibration** → Bound modifiers, test neutral and directional cases over paired seeds, and run the canonical calibration suite with default modifiers unchanged.

## Migration Plan

1. Add typed depth-chart values, canonical templates, validation, and atomic edit APIs.
2. Add deterministic strength calculation and the explicit simulation-modifier adapter.
3. Export the module and add serde, unit, integration, regression, and calibration tests.
4. Document the opt-in boundary and deferred systems.

No stored-data migration is required. Rollback removes the new module and adapter; existing `Team`, roster, and default simulation behavior remain compatible.

## Open Questions

- Should a later scheme capability make slot templates configurable per program, or select from versioned canonical formations?
- When position-specific attributes arrive, should strength formulas be versioned as persisted policy or remain simulation-profile configuration?
