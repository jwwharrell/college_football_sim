//! Deterministic aggregate calibration for the possession simulation.

use crate::simulation::{
    simulate_game, Envelope, Matchup, MatchupModifiers, SimulationConfig, Venue, ALGORITHM_VERSION,
};
use crate::team::Team;
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};

pub const SMOKE_SEED_COUNT: u32 = 40;
pub const CANONICAL_SEED_COUNT: u32 = 1_000;
pub const CANONICAL_SEED_SET: &str = "sequential-0..1000-v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricResult {
    pub name: String,
    pub observed: f64,
    pub expected: Envelope,
    pub sample_size: u64,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub algorithm_version: String,
    pub profile_version: String,
    pub seed_set_identity: String,
    pub seed_count: u32,
    pub sample_size: u64,
    pub matchups: Vec<String>,
    pub metrics: Vec<MetricResult>,
    pub passed: bool,
}

impl CalibrationReport {
    pub fn failures(&self) -> impl Iterator<Item = &MetricResult> {
        self.metrics.iter().filter(|metric| !metric.passed)
    }
}

#[derive(Default)]
struct Totals {
    games: u64,
    team_points: u64,
    team_possessions: u64,
    team_turnovers: u64,
    overtime_games: u64,
    equal_home_games: u64,
    equal_home_wins: u64,
    favorite_games: u64,
    favorite_wins: u64,
}

/// Runs the balanced canonical matchup matrix over sequential fixed seeds.
pub fn run_calibration(config: &SimulationConfig, seed_count: u32) -> SimResult<CalibrationReport> {
    config.validate()?;
    if seed_count == 0 {
        return Err(SimError::InvalidParameter(
            "calibration seed_count must be positive".into(),
        ));
    }
    let equal_home = matchup("equal-home", 75, 75, Venue::Home, false);
    let favorite_first = matchup("favorite-first", 90, 65, Venue::Neutral, false);
    let mut favorite_second = matchup("favorite-second", 65, 90, Venue::Neutral, false);
    favorite_second.home.id = "underdog-home".into();
    favorite_second.away.id = "favorite-away".into();

    let mut totals = Totals::default();
    for seed in 0..u64::from(seed_count) {
        let equal = simulate_game(&equal_home, config, seed)?;
        accumulate(&mut totals, &equal);
        totals.equal_home_games += 1;
        totals.equal_home_wins +=
            u64::from(equal.game.winner().map(|t| t.id.as_str()) == Some("home"));

        let first = simulate_game(&favorite_first, config, seed)?;
        accumulate(&mut totals, &first);
        totals.favorite_games += 1;
        totals.favorite_wins +=
            u64::from(first.game.winner().map(|t| t.id.as_str()) == Some("home"));

        let second = simulate_game(&favorite_second, config, seed)?;
        accumulate(&mut totals, &second);
        totals.favorite_games += 1;
        totals.favorite_wins +=
            u64::from(second.game.winner().map(|t| t.id.as_str()) == Some("favorite-away"));
    }

    let team_games = totals.games * 2;
    let favorite_win_rate = ratio(totals.favorite_wins, totals.favorite_games);
    let values = [
        (
            "points_per_team",
            ratio(totals.team_points, team_games),
            config.envelopes.points_per_team,
            team_games,
        ),
        (
            "possessions_per_team",
            ratio(totals.team_possessions, team_games),
            config.envelopes.possessions_per_team,
            team_games,
        ),
        (
            "turnovers_per_team",
            ratio(totals.team_turnovers, team_games),
            config.envelopes.turnovers_per_team,
            team_games,
        ),
        (
            "overtime_rate",
            ratio(totals.overtime_games, totals.games),
            config.envelopes.overtime_rate,
            totals.games,
        ),
        (
            "equal_team_home_win_rate",
            ratio(totals.equal_home_wins, totals.equal_home_games),
            config.envelopes.equal_team_home_win_rate,
            totals.equal_home_games,
        ),
        (
            "favorite_win_rate_rating_diff_25",
            favorite_win_rate,
            config.envelopes.favorite_win_rate,
            totals.favorite_games,
        ),
        (
            "upset_rate_rating_diff_25",
            ratio(
                totals.favorite_games - totals.favorite_wins,
                totals.favorite_games,
            ),
            config.envelopes.upset_rate,
            totals.favorite_games,
        ),
    ];
    let metrics = values
        .into_iter()
        .map(|(name, observed, expected, sample_size)| MetricResult {
            name: name.into(),
            observed,
            expected,
            sample_size,
            passed: expected.contains(observed),
        })
        .collect::<Vec<_>>();
    let passed = metrics.iter().all(|metric| metric.passed);
    Ok(CalibrationReport {
        algorithm_version: ALGORITHM_VERSION.into(),
        profile_version: config.profile_version.clone(),
        seed_set_identity: if seed_count == CANONICAL_SEED_COUNT {
            CANONICAL_SEED_SET.into()
        } else {
            format!("sequential-0..{seed_count}-v1")
        },
        seed_count,
        sample_size: totals.games,
        matchups: vec![
            "equal-rated teams, non-neutral home venue".into(),
            "rating differential 25, neutral site, favorite designated home".into(),
            "rating differential 25, neutral site, favorite designated away".into(),
        ],
        metrics,
        passed,
    })
}

fn accumulate(totals: &mut Totals, result: &crate::simulation::SimulationResult) {
    totals.games += 1;
    totals.team_points += u64::from(result.home_stats.points + result.away_stats.points);
    totals.team_possessions +=
        u64::from(result.home_stats.possessions + result.away_stats.possessions);
    totals.team_turnovers += u64::from(result.home_stats.turnovers + result.away_stats.turnovers);
    totals.overtime_games += u64::from(result.overtime_rounds > 0);
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn matchup(id: &str, home_rating: u8, away_rating: u8, venue: Venue, conference: bool) -> Matchup {
    Matchup {
        game_id: id.into(),
        home: team("home", home_rating),
        away: team("away", away_rating),
        location: "Calibration Stadium".into(),
        week: 1,
        conference_game: conference,
        venue,
        modifiers: MatchupModifiers::default(),
    }
}

fn team(id: &str, rating: u8) -> Team {
    Team::new(
        id,
        format!("Calibration {id}"),
        id.to_uppercase(),
        "Calibrators",
        "Calibration",
        None,
        "Calibration City",
        rating,
        rating,
        rating,
        rating,
    )
    .expect("calibration fixtures are valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_batch_is_deterministic_and_passes() {
        let config = SimulationConfig::default();
        let first = run_calibration(&config, SMOKE_SEED_COUNT).unwrap();
        let second = run_calibration(&config, SMOKE_SEED_COUNT).unwrap();
        assert_eq!(first, second);
        assert!(
            first.passed,
            "failures: {:?}",
            first.failures().collect::<Vec<_>>()
        );
        assert_eq!(first.sample_size, u64::from(SMOKE_SEED_COUNT) * 3);
    }

    #[test]
    fn report_is_machine_readable_and_diagnostic() {
        let mut config = SimulationConfig::default();
        config.envelopes.points_per_team = Envelope { min: 0.0, max: 0.0 };
        let report = run_calibration(&config, 5).unwrap();
        assert!(!report.passed);
        assert!(report
            .failures()
            .any(|metric| metric.name == "points_per_team"));
        let json = serde_json::to_string(&report).unwrap();
        let decoded: CalibrationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.algorithm_version, decoded.algorithm_version);
        assert_eq!(report.profile_version, decoded.profile_version);
        assert_eq!(report.metrics.len(), decoded.metrics.len());
        assert_eq!(report.passed, decoded.passed);
    }

    #[test]
    #[ignore = "explicit release-sized statistical suite"]
    fn canonical_calibration_passes() {
        let report = run_calibration(&SimulationConfig::default(), CANONICAL_SEED_COUNT).unwrap();
        assert!(
            report.passed,
            "failures: {:?}",
            report.failures().collect::<Vec<_>>()
        );
    }
}
