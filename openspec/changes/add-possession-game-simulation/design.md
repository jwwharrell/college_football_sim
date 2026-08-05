## Context

The Rust workspace already separates pure domain logic (`sim_core`), persistence, and CLI presentation. `Team` exposes overall, offense, defense, and special-teams ratings; `Game` models lifecycle and period scoring; `Season` derives records and standings; and `SimRng` provides seeded randomness. Scores are currently entered by callers, so these pieces do not yet form a simulation.

The intended product is a long-running team-management dynasty game. Simulation therefore needs statistical credibility, deterministic replay, and enough causal detail to explain how future roster, scheme, and staff decisions affect results. The first engine will operate on current team-level ratings, while leaving seams for later player and tactical inputs.

## Goals / Non-Goals

**Goals:**

- Produce a complete, untied football game from two current `Team` values, venue context, configuration, and a seed.
- Model an auditable sequence of possessions with internally consistent scoring and box-score statistics.
- Keep simulation pure, deterministic, versioned, and independent from persistence and presentation.
- Make tuning data explicit and validate behavior over reproducible batches rather than relying only on example games.
- Create stable extension points for future roster units, schemes, coaching, fatigue, weather, and injuries.

**Non-Goals:**

- Play-by-play tactical simulation, clock-management decisions, penalties, individual player statistics, injuries, or fatigue.
- Recruiting, rosters, depth charts, staff, program finances, schedules, rankings, or postseason systems.
- Loading real-world team data or claiming the initial profile reproduces a specific historical season exactly.
- Saving games or calibration reports through the persistence crate.
- Parallel batch execution in the initial implementation.

## Decisions

### Simulate drives rather than quarters or individual plays

The core engine will advance through possessions. A possession samples a play count, duration, yards, and terminal outcome, then updates field position, clock/period, score, and statistics. Outcomes include touchdown, field-goal attempt, punt, turnover, turnover on downs, and end of half/game.

This resolution is detailed enough to generate explainable box scores and later accept tactical modifiers, while remaining far simpler and faster than a full play engine. Quarter-level score sampling was rejected because it cannot reconcile meaningful football statistics or explain management effects. Full play-by-play was deferred because its state space and calibration burden are disproportionate to the current team-only model.

### Separate immutable inputs, mutable simulation state, and public results

The API will conceptually expose:

```text
Matchup + SimulationConfig + seed
                 │
                 ▼
       private GameSimulationState
       clock, field position, possession,
       score, accumulated statistics
                 │
                 ▼
       SimulationResult
       completed Game, possessions,
       summaries, provenance
```

Public result types will use integer counts for source statistics and explicit helper methods for derived rates. Internal state will not leak through the API. This makes invariants testable and avoids partially simulated domain games escaping on failure.

### Derive matchup efficiency from unit ratings

Each offense will be compared primarily with the opposing defense. Overall rating will contribute a small consistency/depth modifier rather than double-counting unit quality, and special-teams rating will affect kicking, punting, and resulting field position. All rating inputs will be normalized to bounded values before coefficients are applied.

Probabilities will be computed from baseline rates plus bounded rating and venue adjustments, then clamped to valid configured ranges. This prevents extreme ratings from creating impossible probabilities. Directly mapping overall rating to final score was rejected because it makes offense, defense, and special teams cosmetic and provides poor extension points for a management game.

### Centralize tuning in a validated, versioned profile

`SimulationConfig` will contain or reference a serializable calibration profile covering baseline drive rates, outcome weights, duration/play/yards distributions, rating coefficients, home advantage, overtime behavior, bounds, and aggregate acceptance envelopes. A checked-in default constructor/profile will be the sole source of production defaults.

Every result will carry an algorithm version constant and profile version. Changing stochastic call order or mechanics requires an algorithm-version change; tuning-only changes require a profile-version change. Exact replay is promised only for the same serialized inputs and both versions.

Hard-coded constants spread across simulation functions were rejected because they make calibration review and comparison unreliable. Runtime file loading inside `sim_core` was rejected because the core must remain pure; callers may eventually deserialize profiles and pass them in.

### Use deterministic substreams for future stability

The simulation will derive domain-specific RNG streams from the supplied seed and stable labels or discriminants, rather than sharing one unconstrained random stream across every concern. At minimum, regulation progression and overtime will be isolated; the design should permit later streams for weather, injuries, or player development.

This reduces unrelated replay changes when a new random draw is introduced. A single sequential RNG was considered simpler, but it makes all downstream possessions change whenever any earlier mechanic consumes an additional value. Stream derivation must use a stable documented mixer, not implementation-dependent hashing.

### Resolve time and period boundaries with explicit rules

Possessions consume integer seconds. A possession ending beyond a period boundary is attributed according to a single documented convention, and the clock advances without negative or fractional time. Halftime resets possession/field context according to configuration. Regulation ends after four quarters.

If regulation is tied, the engine enters alternating overtime possessions using a simplified configurable college format. Each overtime round gives both teams an opportunity unless the configured rules determine the game earlier; rounds repeat until the result is untied. The initial implementation need not reproduce every historical NCAA overtime rule variant.

### Treat aggregate calibration as a first-class test harness

The calibration module will define a matchup matrix containing equal-team home/neutral games and balanced rating-difference bands. It will run fixed canonical seeds, accumulate raw integer totals, derive metrics, and compare them with inclusive profile envelopes.

Two tiers will be used:

- A small deterministic smoke batch in routine `cargo test` for invariants and gross regressions.
- A larger explicit or ignored canonical batch for stable statistical validation before accepting tuning changes.

Initial numeric envelopes are provisional model acceptance criteria, not claims about a particular real season. They should be documented beside the default profile and tightened or shifted using cited datasets in a later data-calibration change. Fixed seed sets plus envelopes were chosen over random seeds and p-value-only tests to eliminate flaky CI.

### Keep CLI and formatting outside the core

The CLI will construct validated sample teams from rating arguments, pass venue/config/seed inputs into `sim_core`, and format the returned result. Batch reporting may expose text output and failure exit status, but no printing or argument parsing belongs in the core.

## Risks / Trade-offs

- **A drive-level abstraction can generate combinations that are statistically plausible but mechanically unusual** → Enforce reconciliation invariants and bounded distributions; add richer field-position mechanics only when observed output justifies it.
- **Current team-level ratings cannot explain player- or scheme-specific outcomes** → Keep matchup modifiers and result types composable so roster units and tactics can be added without replacing the orchestration contract.
- **Calibration envelopes may encode arbitrary realism assumptions** → Label initial targets as provisional, store provenance with profiles, and plan a later dataset-backed calibration change.
- **Large batches can slow tests** → Split fast smoke coverage from an explicit canonical suite and benchmark before choosing final sample counts.
- **Floating-point calculations can threaten cross-platform bit-for-bit replay** → Prefer integer/fixed-point weights and counts in stochastic decisions; if floating point remains necessary, constrain supported replay guarantees and test supported targets.
- **Changing random-call order breaks historical replay** → Use versioned algorithms and stable substreams, and include both versions in every result.
- **Simplified overtime rules will not cover all historical eras** → Make overtime behavior a profile/rules component and explicitly version it.

## Migration Plan

1. Add new simulation and calibration modules without altering existing manual game construction APIs.
2. Introduce validated default configuration and result/provenance types.
3. Add possession orchestration, regulation, overtime, and invariant tests.
4. Add batch analysis and check in the first passing default profile and acceptance report.
5. Add the CLI simulation harness while retaining existing commands.

Rollback consists of removing the new CLI command and modules; existing team, game, season, and manually scored CLI behavior remain compatible. No stored data migration is required.

## Open Questions

- Which public dataset and seasons should become the authoritative source for tightening the provisional calibration envelopes?
- Should long-term replay compatibility be guaranteed across CPU architectures, or only within the supported Rust build targets?
- Which college overtime ruleset should become the first named profile before historical rules are introduced?
