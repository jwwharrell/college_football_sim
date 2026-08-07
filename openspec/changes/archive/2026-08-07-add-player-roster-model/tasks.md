## 1. Player Domain Values

- [x] 1.1 Add `sim_core::player` with serializable stable player identifiers, common position and class-year enums, foundational attribute values, and explicit eligibility/participation types.
- [x] 1.2 Implement validated constructors for player identity, required names, 0–100 attributes, and coherent four-season eligibility state using existing domain errors.
- [x] 1.3 Implement deterministic eligibility advancement for season-used and redshirt outcomes, including exhausted-eligibility and repeated-redshirt failures without partial mutation.
- [x] 1.4 Add player unit tests for valid construction, every identity and rating boundary, eligibility invariants, class-year advancement, stable identity, and error cases.

## 2. Season-Scoped Rosters

- [x] 2.1 Add `sim_core::roster` with validated team identity, positive season year, unique player membership, and deterministic player-ID ordering.
- [x] 2.2 Implement atomic roster construction, lookup, add, and remove operations with duplicate and invalid-player diagnostics.
- [x] 2.3 Implement all-or-nothing next-season transition from explicit per-player participation outcomes, returning the next roster and deterministic eligibility-departure summary.
- [x] 2.4 Add roster unit tests for invalid roots, duplicate membership, insertion-order independence, lookup/removal, missing outcomes, exhausted departures, and source immutability.

## 3. Serialization and Public API

- [x] 3.1 Export player and roster modules from `sim_core` and document that they remain independent of current aggregate `Team` simulation inputs.
- [x] 3.2 Add JSON round-trip and stable snake-case enum tests for players, eligibility, positions, attributes, and canonically ordered rosters.
- [x] 3.3 Add a regression test proving an adjacent roster does not change an existing seeded possession simulation result.

## 4. Documentation and Verification

- [x] 4.1 Update current project documentation to describe the player/roster domain boundary, simplified eligibility policy, and explicitly deferred systems.
- [x] 4.2 Run rustfmt, Clippy with warnings denied, all workspace tests, the canonical simulation calibration suite, and OpenSpec validation.
