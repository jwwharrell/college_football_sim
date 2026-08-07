# Simulation Calibration

The `provisional-cfb-v1` profile defines initial model acceptance criteria. These envelopes are deliberately broad and are not claims about one specific NCAA season. A future data-calibration change should cite an authoritative dataset, select seasons and subdivisions, and revise the profile version and targets together.

## Canonical suite

Run:

```console
cargo run -p cli -- calibrate --seeds 1000
```

The suite uses fixed seeds `0..1000` and three games per seed: equal-rated teams at a home venue, plus a 25-point rating gap at a neutral site with the favorite assigned once to each designation. The 3,000-game debug build completes in approximately 0.3 seconds on the development machine as of the v1 baseline. Runtime is informational and can vary by hardware.

The routine workspace test uses 40 seeds (120 games) to catch gross regressions without making local development slow. The release-sized ignored test can also be run directly:

```console
cargo test -p sim_core calibration::tests::canonical_calibration_passes -- --ignored
```

## Checked-in v1 baseline

- Algorithm: `possession-v1`
- Profile: `provisional-cfb-v1`
- Seed set: `sequential-0..1000-v1`
- Games: 3,000

| Metric | Observed | Accepted envelope |
|---|---:|---:|
| Points per team | 26.1275 | 18.0000–40.0000 |
| Possessions per team | 12.9623 | 8.0000–16.0000 |
| Turnovers per team | 1.4982 | 0.4000–2.5000 |
| Overtime rate | 0.0373 | 0.0000–0.1500 |
| Equal-team home win rate | 0.5190 | 0.5000–0.6800 |
| Favorite win rate, rating gap 25 | 0.8625 | 0.5500–0.9000 |
| Upset rate, rating gap 25 | 0.1375 | 0.1000–0.4500 |

## Updating mechanics or tuning

1. Change `ALGORITHM_VERSION` when mechanics, RNG consumption, or stochastic call ordering changes.
2. Change `profile_version` when only coefficients or acceptance envelopes change.
3. Run formatting, Clippy, the workspace suite, and the canonical calibration suite.
4. Inspect failures before changing targets. Do not widen an envelope merely to conceal a regression.
5. Update this baseline from the machine-readable `calibrate --json` report and document the dataset and rationale behind any target change.

## Interpretation and limits

Passing means the fixed matchup matrix lies within the profile's declared aggregate envelopes; it does not prove realism for every team strength or matchup. The domain now models players and season rosters, but the engine deliberately does not derive simulation inputs from them yet. It still lacks depth charts, player-level effects, schemes, coaching, fatigue, injuries, weather, penalties, and tactical clock management. Public matchup modifiers provide an extension seam for these systems without coupling them to drive orchestration.
