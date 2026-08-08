//! Validated schedules and deterministic regular-season progression.

use crate::simulation::{
    derive_seed, simulate_game, Matchup, MatchupModifiers, SimulationConfig, SimulationResult,
    Venue,
};
use crate::team::Team;
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

const SEASON_GAME_SEED_DOMAIN: &str = "season-game-v1";

/// One immutable game assignment in a regular-season schedule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledGame {
    pub id: String,
    pub home_team_id: String,
    pub away_team_id: String,
    pub location: String,
    pub week: u8,
    pub is_conference_game: bool,
    pub venue: Venue,
}

impl ScheduledGame {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        home_team_id: impl Into<String>,
        away_team_id: impl Into<String>,
        location: impl Into<String>,
        week: u8,
        is_conference_game: bool,
        venue: Venue,
    ) -> Self {
        Self {
            id: id.into(),
            home_team_id: home_team_id.into(),
            away_team_id: away_team_id.into(),
            location: location.into(),
            week,
            is_conference_game,
            venue,
        }
    }
}

/// A validated schedule stored in canonical `(week, game_id)` order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    entries: Vec<ScheduledGame>,
}

impl Schedule {
    pub fn new(entries: Vec<ScheduledGame>, teams: &[Team], total_weeks: u8) -> SimResult<Self> {
        if total_weeks == 0 {
            return Err(SimError::InvalidSchedule(
                "total weeks must be greater than zero".into(),
            ));
        }

        let mut team_ids = BTreeSet::new();
        for team in teams {
            if team.id.trim().is_empty() {
                return Err(SimError::InvalidSchedule(
                    "team identifier cannot be empty".into(),
                ));
            }
            if !team_ids.insert(team.id.as_str()) {
                return Err(SimError::InvalidSchedule(format!(
                    "duplicate team identifier: {}",
                    team.id
                )));
            }
        }

        let mut game_ids = BTreeSet::new();
        let mut weekly_assignments: BTreeMap<(u8, &str), &str> = BTreeMap::new();
        for game in &entries {
            if game.id.trim().is_empty() {
                return Err(SimError::InvalidSchedule(
                    "game identifier cannot be empty".into(),
                ));
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(SimError::InvalidSchedule(format!(
                    "duplicate game identifier: {}",
                    game.id
                )));
            }
            if game.location.trim().is_empty() {
                return Err(SimError::InvalidSchedule(format!(
                    "game {} location cannot be empty",
                    game.id
                )));
            }
            if game.week == 0 || game.week > total_weeks {
                return Err(SimError::InvalidSchedule(format!(
                    "game {} has invalid week {} (expected 1..={total_weeks})",
                    game.id, game.week
                )));
            }
            if game.home_team_id == game.away_team_id {
                return Err(SimError::InvalidSchedule(format!(
                    "game {} cannot match team {} against itself",
                    game.id, game.home_team_id
                )));
            }
            for team_id in [&game.home_team_id, &game.away_team_id] {
                if !team_ids.contains(team_id.as_str()) {
                    return Err(SimError::InvalidSchedule(format!(
                        "game {} references unknown team {}",
                        game.id, team_id
                    )));
                }
                if let Some(other_game) = weekly_assignments.insert((game.week, team_id), &game.id)
                {
                    return Err(SimError::InvalidSchedule(format!(
                        "team {team_id} is assigned to games {other_game} and {} in week {}",
                        game.id, game.week
                    )));
                }
            }
        }

        let mut entries = entries;
        entries.sort_by(|left, right| (left.week, &left.id).cmp(&(right.week, &right.id)));
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[ScheduledGame] {
        &self.entries
    }

    pub fn games_for_week(&self, week: u8) -> impl Iterator<Item = &ScheduledGame> {
        self.entries.iter().filter(move |game| game.week == week)
    }

    pub fn games_for_team<'a>(
        &'a self,
        team_id: &'a str,
    ) -> impl Iterator<Item = &'a ScheduledGame> {
        self.entries
            .iter()
            .filter(move |game| game.home_team_id == team_id || game.away_team_id == team_id)
    }

    pub fn game(&self, game_id: &str) -> Option<&ScheduledGame> {
        self.entries.iter().find(|game| game.id == game_id)
    }
}

/// A team's derived regular-season record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamRecord {
    pub wins: u8,
    pub losses: u8,
    pub ties: u8,
    pub conference_wins: u8,
    pub conference_losses: u8,
    pub conference_ties: u8,
}

impl TeamRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn winning_percentage(&self) -> f64 {
        percentage(self.wins, self.losses, self.ties)
    }

    pub fn conference_winning_percentage(&self) -> f64 {
        percentage(
            self.conference_wins,
            self.conference_losses,
            self.conference_ties,
        )
    }

    pub fn conference_to_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.conference_wins, self.conference_losses, self.conference_ties
        )
    }
}

impl fmt::Display for TeamRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}", self.wins, self.losses, self.ties)
    }
}

/// Complete serializable state for a regular season.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub year: u16,
    pub teams: Vec<Team>,
    pub total_weeks: u8,
    current_week: u8,
    schedule: Schedule,
    completed_results: BTreeMap<String, SimulationResult>,
    team_records: BTreeMap<String, TeamRecord>,
}

impl Season {
    pub fn new(
        year: u16,
        teams: Vec<Team>,
        total_weeks: u8,
        entries: Vec<ScheduledGame>,
    ) -> SimResult<Self> {
        let schedule = Schedule::new(entries, &teams, total_weeks)?;
        let team_records = teams
            .iter()
            .map(|team| (team.id.clone(), TeamRecord::new()))
            .collect();
        Ok(Self {
            year,
            teams,
            total_weeks,
            current_week: 1,
            schedule,
            completed_results: BTreeMap::new(),
            team_records,
        })
    }

    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// The active week, or `None` after the regular season is complete.
    pub fn current_week(&self) -> Option<u8> {
        (!self.is_complete()).then_some(self.current_week)
    }

    pub fn current_week_games(&self) -> impl Iterator<Item = &ScheduledGame> {
        self.schedule.games_for_week(self.current_week)
    }

    pub fn completed_results(&self) -> &BTreeMap<String, SimulationResult> {
        &self.completed_results
    }

    pub fn result_for_game(&self, game_id: &str) -> Option<&SimulationResult> {
        self.completed_results.get(game_id)
    }

    pub fn record_for_team(&self, team_id: &str) -> Option<&TeamRecord> {
        self.team_records.get(team_id)
    }

    pub fn is_complete(&self) -> bool {
        self.current_week > self.total_weeks
    }

    pub fn game_seed(&self, season_seed: u64, game_id: &str) -> SimResult<u64> {
        if self.schedule.game(game_id).is_none() {
            return Err(SimError::GameNotFound(game_id.into()));
        }
        Ok(derive_season_game_seed(season_seed, self.year, game_id))
    }

    pub fn matchup_for_game(&self, game_id: &str) -> SimResult<Matchup> {
        let scheduled = self
            .schedule
            .game(game_id)
            .ok_or_else(|| SimError::GameNotFound(game_id.into()))?;
        let teams: HashMap<&str, &Team> = self
            .teams
            .iter()
            .map(|team| (team.id.as_str(), team))
            .collect();
        let home = teams
            .get(scheduled.home_team_id.as_str())
            .ok_or_else(|| SimError::TeamNotFound(scheduled.home_team_id.clone()))?;
        let away = teams
            .get(scheduled.away_team_id.as_str())
            .ok_or_else(|| SimError::TeamNotFound(scheduled.away_team_id.clone()))?;
        Ok(Matchup {
            game_id: scheduled.id.clone(),
            home: (*home).clone(),
            away: (*away).clone(),
            location: scheduled.location.clone(),
            week: scheduled.week,
            conference_game: scheduled.is_conference_game,
            venue: scheduled.venue,
            modifiers: MatchupModifiers::default(),
        })
    }

    /// Simulates and commits the active week as one all-or-nothing transition.
    pub fn advance_week(
        &mut self,
        season_seed: u64,
        config: &SimulationConfig,
    ) -> SimResult<Vec<String>> {
        if self.is_complete() {
            return Err(SimError::SeasonComplete);
        }
        config.validate()?;

        let games = self
            .schedule
            .games_for_week(self.current_week)
            .cloned()
            .collect::<Vec<_>>();
        if let Some(game) = games
            .iter()
            .find(|game| self.completed_results.contains_key(&game.id))
        {
            return Err(SimError::InvalidSeasonState(format!(
                "week {} already contains a result for game {}",
                self.current_week, game.id
            )));
        }

        let mut pending = Vec::with_capacity(games.len());
        for game in &games {
            let matchup = self.matchup_for_game(&game.id)?;
            let seed = self.game_seed(season_seed, &game.id)?;
            let result = simulate_game(&matchup, config, seed)?;
            pending.push((game.id.clone(), result));
        }

        let committed_ids = pending.iter().map(|(id, _)| id.clone()).collect();
        let mut candidate = self.clone();
        for (game_id, result) in pending {
            candidate.completed_results.insert(game_id, result);
        }
        candidate.rebuild_records()?;
        candidate.current_week = candidate
            .current_week
            .checked_add(1)
            .ok_or_else(|| SimError::InvalidSeasonState("week counter overflow".into()))?;
        *self = candidate;
        Ok(committed_ids)
    }

    pub fn advance_regular_season(
        &mut self,
        season_seed: u64,
        config: &SimulationConfig,
    ) -> SimResult<()> {
        while !self.is_complete() {
            self.advance_week(season_seed, config)?;
        }
        Ok(())
    }

    pub fn conference_standings(&self, conference: &str) -> Vec<(&Team, TeamRecord)> {
        self.sorted_standings(|team| team.conference == conference)
    }

    pub fn division_standings(&self, conference: &str, division: &str) -> Vec<(&Team, TeamRecord)> {
        self.sorted_standings(|team| {
            team.conference == conference && team.division.as_deref() == Some(division)
        })
    }

    fn sorted_standings(&self, include: impl Fn(&Team) -> bool) -> Vec<(&Team, TeamRecord)> {
        let mut standings = self
            .teams
            .iter()
            .filter(|team| include(team))
            .filter_map(|team| {
                self.team_records
                    .get(&team.id)
                    .map(|record| (team, *record))
            })
            .collect::<Vec<_>>();
        standings.sort_by(|(left_team, left), (right_team, right)| {
            compare_records(right, left).then_with(|| left_team.id.cmp(&right_team.id))
        });
        standings
    }

    fn rebuild_records(&mut self) -> SimResult<()> {
        let mut records = self
            .teams
            .iter()
            .map(|team| (team.id.clone(), TeamRecord::new()))
            .collect::<BTreeMap<_, _>>();
        for (game_id, result) in &self.completed_results {
            let scheduled = self.schedule.game(game_id).ok_or_else(|| {
                SimError::InvalidSeasonState(format!(
                    "completed result {game_id} has no schedule entry"
                ))
            })?;
            let home_score = result.game.home_score.total;
            let away_score = result.game.away_score.total;
            match home_score.cmp(&away_score) {
                Ordering::Greater => update_result(
                    &mut records,
                    &scheduled.home_team_id,
                    &scheduled.away_team_id,
                    scheduled.is_conference_game,
                )?,
                Ordering::Less => update_result(
                    &mut records,
                    &scheduled.away_team_id,
                    &scheduled.home_team_id,
                    scheduled.is_conference_game,
                )?,
                Ordering::Equal => {
                    return Err(SimError::InvalidSeasonState(format!(
                        "completed simulation {game_id} is tied"
                    )))
                }
            }
        }
        self.team_records = records;
        Ok(())
    }
}

/// Derives an isolated seed with unambiguous, versioned identity fields.
pub fn derive_season_game_seed(season_seed: u64, year: u16, game_id: &str) -> u64 {
    let label = format!(
        "{SEASON_GAME_SEED_DOMAIN}|year:4:{year:04}|game:{}:{game_id}",
        game_id.len()
    );
    derive_seed(season_seed, &label)
}

fn percentage(wins: u8, losses: u8, ties: u8) -> f64 {
    let games = f64::from(wins) + f64::from(losses) + f64::from(ties);
    if games == 0.0 {
        0.0
    } else {
        (f64::from(wins) + 0.5 * f64::from(ties)) / games
    }
}

fn compare_records(left: &TeamRecord, right: &TeamRecord) -> Ordering {
    left.conference_winning_percentage()
        .total_cmp(&right.conference_winning_percentage())
        .then_with(|| {
            left.winning_percentage()
                .total_cmp(&right.winning_percentage())
        })
        .then_with(|| left.conference_wins.cmp(&right.conference_wins))
        .then_with(|| left.wins.cmp(&right.wins))
}

fn update_result(
    records: &mut BTreeMap<String, TeamRecord>,
    winner_id: &str,
    loser_id: &str,
    conference: bool,
) -> SimResult<()> {
    let winner = records
        .get_mut(winner_id)
        .ok_or_else(|| SimError::TeamNotFound(winner_id.into()))?;
    winner.wins = winner
        .wins
        .checked_add(1)
        .ok_or_else(|| SimError::InvalidSeasonState("winner record overflow".into()))?;
    if conference {
        winner.conference_wins = winner.conference_wins.checked_add(1).ok_or_else(|| {
            SimError::InvalidSeasonState("conference winner record overflow".into())
        })?;
    }

    let loser = records
        .get_mut(loser_id)
        .ok_or_else(|| SimError::TeamNotFound(loser_id.into()))?;
    loser.losses = loser
        .losses
        .checked_add(1)
        .ok_or_else(|| SimError::InvalidSeasonState("loser record overflow".into()))?;
    if conference {
        loser.conference_losses = loser.conference_losses.checked_add(1).ok_or_else(|| {
            SimError::InvalidSeasonState("conference loser record overflow".into())
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(id: &str, conference: &str, rating: u8) -> Team {
        Team::new(
            id,
            format!("Team {id}"),
            id.to_uppercase(),
            format!("{id}s"),
            conference,
            None,
            format!("{id} City"),
            rating,
            rating,
            rating,
            rating,
        )
        .unwrap()
    }

    fn game(id: &str, home: &str, away: &str, week: u8, conference: bool) -> ScheduledGame {
        ScheduledGame::new(
            id,
            home,
            away,
            format!("{home} Stadium"),
            week,
            conference,
            Venue::Home,
        )
    }

    fn teams() -> Vec<Team> {
        vec![
            team("a", "Alpha", 84),
            team("b", "Alpha", 76),
            team("c", "Alpha", 70),
            team("d", "Other", 68),
        ]
    }

    #[test]
    fn schedule_is_validated_canonical_and_queryable() {
        let entries = vec![
            game("z-week-two", "a", "d", 2, false),
            game("b-week-one", "c", "d", 1, false),
            game("a-week-one", "a", "b", 1, true),
        ];
        let schedule = Schedule::new(entries, &teams(), 3).unwrap();
        let ids = schedule
            .entries()
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["a-week-one", "b-week-one", "z-week-two"]);
        assert_eq!(schedule.games_for_week(1).count(), 2);
        assert_eq!(schedule.games_for_team("a").count(), 2);
        assert_eq!(schedule.games_for_team("c").count(), 1);
        let entry = schedule.game("a-week-one").unwrap();
        assert_eq!(entry.location, "a Stadium");
        assert!(entry.is_conference_game);
        assert_eq!(entry.venue, Venue::Home);
    }

    #[test]
    fn schedule_allows_byes_and_empty_weeks() {
        let schedule = Schedule::new(vec![game("one", "a", "b", 1, true)], &teams(), 3).unwrap();
        assert_eq!(schedule.games_for_team("c").count(), 0);
        assert_eq!(schedule.games_for_week(2).count(), 0);
    }

    #[test]
    fn schedule_rejects_each_invalid_shape() {
        let cases = [
            vec![game("", "a", "b", 1, true)],
            vec![
                game("same", "a", "b", 1, true),
                game("same", "c", "d", 1, false),
            ],
            vec![game("unknown", "a", "missing", 1, false)],
            vec![game("self", "a", "a", 1, false)],
            vec![game("zero", "a", "b", 0, false)],
            vec![game("late", "a", "b", 4, false)],
            vec![
                game("one", "a", "b", 1, true),
                game("two", "a", "c", 1, true),
            ],
        ];
        for entries in cases {
            assert!(matches!(
                Schedule::new(entries, &teams(), 3),
                Err(SimError::InvalidSchedule(_))
            ));
        }
    }

    #[test]
    fn season_rejects_duplicate_team_ids() {
        let duplicate = vec![team("a", "Alpha", 80), team("a", "Alpha", 70)];
        assert!(matches!(
            Season::new(2026, duplicate, 1, Vec::new()),
            Err(SimError::InvalidSchedule(_))
        ));
    }

    #[test]
    fn matchup_preserves_schedule_metadata() {
        let mut neutral = game("neutral", "a", "b", 2, true);
        neutral.location = "Bowl Site".into();
        neutral.venue = Venue::Neutral;
        let season = Season::new(2026, teams(), 2, vec![neutral]).unwrap();
        let matchup = season.matchup_for_game("neutral").unwrap();
        assert_eq!(matchup.home.id, "a");
        assert_eq!(matchup.away.id, "b");
        assert_eq!(matchup.location, "Bowl Site");
        assert_eq!(matchup.week, 2);
        assert!(matchup.conference_game);
        assert_eq!(matchup.venue, Venue::Neutral);
        assert_eq!(matchup.modifiers, MatchupModifiers::default());
    }

    #[test]
    fn seed_derivation_is_stable_isolated_and_unambiguous() {
        let seed = derive_season_game_seed(42, 2026, "game-a");
        assert_eq!(seed, derive_season_game_seed(42, 2026, "game-a"));
        assert_ne!(seed, derive_season_game_seed(42, 2026, "game-b"));
        assert_ne!(seed, derive_season_game_seed(42, 2027, "game-a"));
        assert_ne!(
            derive_season_game_seed(42, 2026, "1:a"),
            derive_season_game_seed(42, 2026, "1:a|extra")
        );
    }

    #[test]
    fn weekly_progression_is_deterministic_and_order_independent() {
        let entries = vec![
            game("second", "c", "d", 1, false),
            game("first", "a", "b", 1, true),
        ];
        let mut reversed = entries.clone();
        reversed.reverse();
        let mut left = Season::new(2026, teams(), 1, entries).unwrap();
        let mut right = Season::new(2026, teams(), 1, reversed).unwrap();
        left.advance_week(9001, &SimulationConfig::default())
            .unwrap();
        right
            .advance_week(9001, &SimulationConfig::default())
            .unwrap();
        for id in ["first", "second"] {
            let left = left.result_for_game(id).unwrap();
            let right = right.result_for_game(id).unwrap();
            assert_eq!(left.provenance, right.provenance);
            assert_eq!(left.game.home_score, right.game.home_score);
            assert_eq!(left.game.away_score, right.game.away_score);
            assert_eq!(left.possessions, right.possessions);
        }
    }

    #[test]
    fn unrelated_later_game_does_not_change_existing_result() {
        let first = game("first", "a", "b", 1, true);
        let mut base = Season::new(2026, teams(), 2, vec![first.clone()]).unwrap();
        let mut extended = Season::new(
            2026,
            teams(),
            2,
            vec![first, game("later", "c", "d", 2, false)],
        )
        .unwrap();
        base.advance_week(77, &SimulationConfig::default()).unwrap();
        extended
            .advance_week(77, &SimulationConfig::default())
            .unwrap();
        let left = base.result_for_game("first").unwrap();
        let right = extended.result_for_game("first").unwrap();
        assert_eq!(left.provenance.seed, right.provenance.seed);
        assert_eq!(left.game.home_score, right.game.home_score);
        assert_eq!(left.game.away_score, right.game.away_score);
    }

    #[test]
    fn failed_week_is_atomic_and_completed_season_cannot_advance() {
        let mut season =
            Season::new(2026, teams(), 1, vec![game("one", "a", "b", 1, true)]).unwrap();
        let mut invalid = SimulationConfig::default();
        invalid.profile_version.clear();
        assert!(season.advance_week(1, &invalid).is_err());
        assert_eq!(season.current_week(), Some(1));
        assert!(season.completed_results().is_empty());
        assert_eq!(*season.record_for_team("a").unwrap(), TeamRecord::new());

        season
            .advance_week(1, &SimulationConfig::default())
            .unwrap();
        assert!(season.is_complete());
        assert_eq!(season.current_week(), None);
        assert!(matches!(
            season.advance_week(1, &SimulationConfig::default()),
            Err(SimError::SeasonComplete)
        ));
        assert_eq!(season.completed_results().len(), 1);
    }

    #[test]
    fn later_game_failure_and_partial_current_week_are_atomic() {
        let entries = vec![
            game("first", "a", "b", 1, true),
            game("second", "c", "d", 1, false),
        ];
        let mut invalid_team = Season::new(2026, teams(), 1, entries.clone()).unwrap();
        invalid_team
            .teams
            .iter_mut()
            .find(|team| team.id == "d")
            .unwrap()
            .rating = 101;
        assert!(invalid_team
            .advance_week(5, &SimulationConfig::default())
            .is_err());
        assert_eq!(invalid_team.current_week(), Some(1));
        assert!(invalid_team.completed_results().is_empty());

        let mut source = Season::new(2026, teams(), 1, entries).unwrap();
        source
            .advance_week(5, &SimulationConfig::default())
            .unwrap();
        let mut partial = Season::new(
            2026,
            teams(),
            1,
            vec![
                game("first", "a", "b", 1, true),
                game("second", "c", "d", 1, false),
            ],
        )
        .unwrap();
        partial.completed_results.insert(
            "first".into(),
            source.result_for_game("first").unwrap().clone(),
        );
        assert!(matches!(
            partial.advance_week(5, &SimulationConfig::default()),
            Err(SimError::InvalidSeasonState(_))
        ));
        assert_eq!(partial.current_week(), Some(1));
        assert_eq!(partial.completed_results().len(), 1);
    }

    #[test]
    fn empty_week_advances_once() {
        let mut season =
            Season::new(2026, teams(), 2, vec![game("later", "a", "b", 2, true)]).unwrap();
        let ids = season
            .advance_week(1, &SimulationConfig::default())
            .unwrap();
        assert!(ids.is_empty());
        assert_eq!(season.current_week(), Some(2));
        assert!(season.completed_results().is_empty());
    }

    #[test]
    fn full_season_preserves_results_records_and_provenance() {
        let entries = vec![
            game("conference", "a", "b", 1, true),
            game("nonconference", "a", "d", 2, false),
            game("other", "b", "c", 2, true),
        ];
        let mut season = Season::new(2026, teams(), 3, entries).unwrap();
        season
            .advance_regular_season(123, &SimulationConfig::default())
            .unwrap();
        assert!(season.is_complete());
        assert_eq!(season.current_week(), None);
        assert_eq!(season.schedule().entries().len(), 3);
        assert_eq!(season.completed_results().len(), 3);
        assert!(season.completed_results().values().all(|result| result
            .provenance
            .algorithm_version
            == "possession-v1"
            && result.provenance.profile_version == "provisional-cfb-v1"));
        let total_games: u8 = season
            .teams
            .iter()
            .map(|team| {
                let record = season.record_for_team(&team.id).unwrap();
                record.wins + record.losses
            })
            .sum();
        assert_eq!(total_games, 6);
        let conference_games: u8 = season
            .teams
            .iter()
            .map(|team| {
                let record = season.record_for_team(&team.id).unwrap();
                record.conference_wins + record.conference_losses
            })
            .sum();
        assert_eq!(conference_games, 4);
    }

    #[test]
    fn standings_are_stable_and_records_are_not_double_counted() {
        let mut season =
            Season::new(2026, teams(), 2, vec![game("one", "a", "b", 1, true)]).unwrap();
        season
            .advance_week(8, &SimulationConfig::default())
            .unwrap();
        let before = *season.record_for_team("a").unwrap();
        season
            .advance_week(8, &SimulationConfig::default())
            .unwrap();
        assert_eq!(before, *season.record_for_team("a").unwrap());

        let standings = season.conference_standings("Alpha");
        assert_eq!(standings.len(), 3);
        let tied_unplayed = standings
            .iter()
            .filter(|(_, record)| *record == TeamRecord::new())
            .map(|(team, _)| team.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(tied_unplayed, ["c"]);

        let no_games = Season::new(2026, teams(), 1, Vec::new()).unwrap();
        let ids = no_games
            .conference_standings("Alpha")
            .iter()
            .map(|(team, _)| team.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn season_state_round_trips_with_full_results() {
        let mut season =
            Season::new(2026, teams(), 1, vec![game("one", "a", "b", 1, true)]).unwrap();
        season
            .advance_week(55, &SimulationConfig::default())
            .unwrap();
        let json = serde_json::to_string(&season).unwrap();
        let restored: Season = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.result_for_game("one").unwrap().provenance.seed,
            season.result_for_game("one").unwrap().provenance.seed
        );
        assert_eq!(restored.record_for_team("a"), season.record_for_team("a"));
        assert_eq!(restored.schedule().entries(), season.schedule().entries());
    }
}
