## Context

`sim_core` already has a pure, seeded possession simulator and basic `Season`/`Game` types, but the season model is permissive: callers append games, manually update records, and increment a week counter independently. It does not validate a schedule, retain full simulation results, isolate per-game seeds, or make weekly advancement transactional. The CLI can exercise one matchup but cannot run a dynasty regular season.

This change is the first orchestration layer above SIM-01. It must preserve the workspace boundary in which `sim_core` performs pure domain transitions, persistence is an adapter concern, and the CLI owns parsing and output. Serialized season state must remain reproducible for future save/load work, even though persistence itself is not part of SEASON-01.

## Goals / Non-Goals

**Goals:**

- Represent and validate a complete regular-season schedule with stable identities and deterministic ordering.
- Advance one week as an atomic pure-domain transition using the existing possession simulator.
- Make every game reproducible independently of schedule collection order.
- Commit completed simulation detail and provenance, then derive consistent records and standings.
- Provide a CLI harness for schedule inspection, one-week advancement, and full-season advancement.

**Non-Goals:**

- Generating an optimized conference schedule or enforcing real-world conference rotation rules.
- Rankings, polls, tiebreakers that select champions, conference championships, bowls, or playoffs.
- Persistence adapters, save migration, recruiting, injuries, fatigue, or between-week roster development.
- Parallel game simulation or a user-facing dynasty UI.

## Decisions

### Separate schedule definitions from completed simulation results

Introduce a compact serializable schedule entry containing stable game ID, home/away team IDs, week, location, conference flag, and venue. The season owns teams once and resolves IDs when constructing a `Matchup`; it does not duplicate mutable team snapshots throughout the schedule. Completed results are stored by game ID and retain the existing `SimulationResult`, including box score and provenance.

Using the current `Game` as both intent and result was considered, but it embeds cloned teams and permits lifecycle mutation that can diverge from the schedule. A separate immutable schedule definition makes validation and replay inputs explicit while allowing the existing result model to remain the simulation output.

### Validate the whole schedule at season construction

A validated constructor checks unique non-empty team and game IDs, non-self matchups, team membership, week bounds, and at most one game per team per week. It canonicalizes schedule access by `(week, game_id)` while allowing byes and weeks without games. Public mutation that could bypass these invariants will be removed, narrowed, or made fallible.

Incrementally accepting arbitrary games was rejected because errors would surface mid-dynasty and make atomic advancement harder. Schedule optimization is deliberately absent: callers remain responsible for choosing opponents and weeks.

### Model advancement as a candidate-state transaction

`advance_week` will take immutable advancement inputs (season seed, validated `SimulationConfig`, and currently neutral/default matchup modifiers), validate current state, build and simulate all current-week matchups into temporary results, recompute records on a candidate season, increment the week, and return/commit the candidate only after every operation succeeds. The precise Rust shape may be a consuming `Result<Season, _>` transition or clone-and-swap method, but failure must leave the caller's original value observably unchanged.

Mutating each game as it finishes was rejected because a later error would expose a partially completed week. A database transaction is also inappropriate because `sim_core` must remain pure and persistence is not yet in scope.

### Derive seeds from length-delimited stable identity data

Each game seed will use the existing stable seed derivation primitive with a versioned, unambiguous label built from season year and game ID (for example, length-delimited fields under a `season-game-v1` domain). Games are processed in canonical order for stable output, but their random streams do not depend on that order. The stored `SimulationResult` supplies the derived seed, algorithm version, and profile version needed for replay.

Drawing game seeds sequentially from one season RNG was rejected because inserting or reordering a matchup would perturb unrelated results. Rust's default hashers are not suitable because their stability is not guaranteed.

### Derive records from committed results instead of applying deltas

After a successful week, overall and conference records are rebuilt from all committed completed results. This makes record calculation idempotent and avoids double-counting if APIs evolve. Standings use documented comparison keys and team ID as a final ascending fallback so ties always have a total order; this fallback is not a competitive tiebreaker.

Applying incremental win/loss mutations is cheaper, but seasons are small and recomputation is safer. Rankings and championship-selection logic remain separate future capabilities.

### Keep orchestration in `sim_core` and examples in the CLI

The core will expose schedule construction, week queries, derived-seed helpers as appropriate, and week/season progression without I/O. CLI subcommands will construct a deterministic sample or supplied schedule, choose the default simulation profile, invoke core APIs, and format schedule/results/standings. No new dependency is needed.

Putting the loop directly in the CLI was rejected because persistence and a future GUI would then duplicate domain invariants and transaction behavior.

## Risks / Trade-offs

- **Retaining full possession results increases serialized season size** → Keep results structurally shared only where practical and defer compaction until persistence requirements provide real constraints.
- **Changing the permissive `Season` API may break current callers and tests** → Introduce validated constructors and migrate the small internal/CLI call surface in one change; keep compatibility helpers only if they cannot violate invariants.
- **Team ratings can change later in a dynasty, complicating historical replay** → Completed results retain provenance, while future persistence must snapshot or version matchup inputs before roster development is added.
- **Identifier-derived seeds make renaming a game result-changing** → Treat game IDs as durable schedule identity and test this contract explicitly.
- **Atomic clone-and-swap can copy a season's accumulated results** → Prefer a consuming transition or temporary current-week result buffer if profiling shows a cost; correctness takes priority for the initial scale.
- **Simple standings ordering may look like a sports tiebreaker** → Document it as deterministic presentation only and keep championship/ranking semantics out of the API.

## Migration Plan

1. Add validated schedule types and season construction while preserving the existing simulator API.
2. Add candidate-state weekly advancement, result/provenance storage, and deterministic record/standings derivation.
3. Migrate existing season tests and CLI callers away from permissive mutation.
4. Add CLI schedule and season-loop commands plus deterministic end-to-end tests.
5. Run formatting, unit tests, Clippy, and repeated-seed acceptance checks.

Rollback removes the new CLI commands and progression/schedule APIs and restores the prior season call sites. There is no stored-data migration because persistence is out of scope.

## Open Questions

- Should the eventual persistence format snapshot the exact pregame team/matchup inputs in addition to the full simulation result, or version team state separately?
- Which future capability will supply non-default roster, injury, fatigue, and coaching modifiers to each scheduled matchup?
