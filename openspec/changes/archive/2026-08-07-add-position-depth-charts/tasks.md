## 1. Typed Depth-Chart Domain

- [x] 1.1 Add `sim_core::depth_chart` with serializable unit, canonical slot, ordinal, assignment, and chart value types using stable snake-case representations.
- [x] 1.2 Define the canonical offense, defense, and special-teams starter templates plus explicit slot-to-`Position` compatibility rules, including restricted specialist slots.
- [x] 1.3 Implement validated chart construction against roster team/season identity, membership, remaining eligibility, required starters, slot compatibility, and per-unit player uniqueness.
- [x] 1.4 Implement deterministic chart lookup/iteration and atomic assign, reorder, and remove operations that preserve a valid complete chart or leave the source unchanged.
- [x] 1.5 Add domain tests for complete and incomplete charts, roster mismatch, missing/exhausted players, compatible/incompatible positions, athlete specialist rejection, duplicate use, cross-unit overlap, depth order, and edit atomicity.

## 2. Deterministic Unit Strengths

- [x] 2.1 Define documented integer attribute, slot, and decreasing depth weights for canonical offense, defense, and special-teams assignments.
- [x] 2.2 Implement pure 0–100 offense, defense, and special-teams strength calculation with deterministic rounding and bounds.
- [x] 2.3 Add strength tests for reproducibility, boundary values, monotonic starter improvement, backup weighting, and insertion-order independence.

## 3. Simulation Composition

- [x] 3.1 Implement a pure adapter from unit strengths to bounded `MatchupModifiers`, with strength 50 producing neutral modifiers and aggregate `Team` ratings remaining baseline inputs.
- [x] 3.2 Add integration tests proving identical roster-derived inputs replay exactly and stronger offense, defense, and special teams have the required directional effects over paired seeds.
- [x] 3.3 Add regression tests proving callers that omit depth-chart modifiers retain existing seeded simulation results and default calibration behavior.

## 4. Serialization, API, and Documentation

- [x] 4.1 Export the depth-chart API from `sim_core` and add validated JSON round-trip tests for charts, stable enum names, canonical unit/slot ordering, and preserved player depth order.
- [x] 4.2 Update current project documentation with the manager-controlled depth-chart workflow, strength formula/version boundary, opt-in simulation integration, issue #15 linkage, and explicitly deferred live personnel systems.

## 5. Verification

- [x] 5.1 Run rustfmt, Clippy with warnings denied, all workspace tests, the canonical 1,000-seed calibration suite, and strict OpenSpec validation.
