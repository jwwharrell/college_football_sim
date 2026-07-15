//! Season module for the college football simulator.
//!
//! This module defines the Season type and related functionality.

use crate::game::{Game, GameStatus};
use crate::team::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a team's record in a season.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamRecord {
    /// Number of wins
    pub wins: u8,
    /// Number of losses
    pub losses: u8,
    /// Number of ties
    pub ties: u8,
    /// Number of conference wins
    pub conference_wins: u8,
    /// Number of conference losses
    pub conference_losses: u8,
    /// Number of conference ties
    pub conference_ties: u8,
}

impl TeamRecord {
    /// Creates a new TeamRecord with all values set to 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the winning percentage (0.0 to 1.0).
    pub fn winning_percentage(&self) -> f64 {
        let total_games = self.wins as f64 + self.losses as f64 + self.ties as f64;
        if total_games == 0.0 {
            return 0.0;
        }
        (self.wins as f64 + 0.5 * self.ties as f64) / total_games
    }

    /// Returns the conference winning percentage (0.0 to 1.0).
    pub fn conference_winning_percentage(&self) -> f64 {
        let total_games = self.conference_wins as f64
            + self.conference_losses as f64
            + self.conference_ties as f64;
        if total_games == 0.0 {
            return 0.0;
        }
        (self.conference_wins as f64 + 0.5 * self.conference_ties as f64) / total_games
    }

    /// Returns the conference record as a string (e.g., "7-1-0").
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

/// Represents a college football season.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    /// Year of the season
    pub year: u16,
    /// List of teams participating in the season
    pub teams: Vec<Team>,
    /// List of games in the season
    pub games: Vec<Game>,
    /// Current week of the season (1-indexed)
    pub current_week: u8,
    /// Total number of weeks in the regular season
    pub total_weeks: u8,
    /// Team records, keyed by team ID
    pub team_records: HashMap<String, TeamRecord>,
}

impl Season {
    /// Creates a new season with the given year and teams.
    pub fn new(year: u16, teams: Vec<Team>, total_weeks: u8) -> Self {
        let mut team_records = HashMap::new();
        for team in &teams {
            team_records.insert(team.id.clone(), TeamRecord::new());
        }

        Self {
            year,
            teams,
            games: Vec::new(),
            current_week: 1,
            total_weeks,
            team_records,
        }
    }

    /// Adds a game to the season.
    pub fn add_game(&mut self, game: Game) {
        self.games.push(game);
    }

    /// Returns all games for a specific week.
    pub fn games_for_week(&self, week: u8) -> Vec<&Game> {
        self.games.iter().filter(|g| g.week == week).collect()
    }

    /// Returns all games for the current week.
    pub fn current_week_games(&self) -> Vec<&Game> {
        self.games_for_week(self.current_week)
    }

    /// Returns all games for a specific team.
    pub fn games_for_team(&self, team_id: &str) -> Vec<&Game> {
        self.games
            .iter()
            .filter(|g| g.home_team.id == team_id || g.away_team.id == team_id)
            .collect()
    }

    /// Returns the record for a specific team.
    pub fn record_for_team(&self, team_id: &str) -> Option<&TeamRecord> {
        self.team_records.get(team_id)
    }

    /// Updates the team records based on completed games.
    pub fn update_records(&mut self) {
        // Reset all records
        for record in self.team_records.values_mut() {
            *record = TeamRecord::new();
        }

        // Update records based on completed games
        for game in &self.games {
            if game.status != GameStatus::Completed {
                continue;
            }

            let home_id = &game.home_team.id;
            let away_id = &game.away_team.id;

            match game.home_score.total.cmp(&game.away_score.total) {
                std::cmp::Ordering::Greater => {
                    // Home team wins
                    if let Some(record) = self.team_records.get_mut(home_id) {
                        record.wins += 1;
                        if game.is_conference_game {
                            record.conference_wins += 1;
                        }
                    }
                    if let Some(record) = self.team_records.get_mut(away_id) {
                        record.losses += 1;
                        if game.is_conference_game {
                            record.conference_losses += 1;
                        }
                    }
                }
                std::cmp::Ordering::Less => {
                    // Away team wins
                    if let Some(record) = self.team_records.get_mut(home_id) {
                        record.losses += 1;
                        if game.is_conference_game {
                            record.conference_losses += 1;
                        }
                    }
                    if let Some(record) = self.team_records.get_mut(away_id) {
                        record.wins += 1;
                        if game.is_conference_game {
                            record.conference_wins += 1;
                        }
                    }
                }
                std::cmp::Ordering::Equal => {
                    // Tie
                    if let Some(record) = self.team_records.get_mut(home_id) {
                        record.ties += 1;
                        if game.is_conference_game {
                            record.conference_ties += 1;
                        }
                    }
                    if let Some(record) = self.team_records.get_mut(away_id) {
                        record.ties += 1;
                        if game.is_conference_game {
                            record.conference_ties += 1;
                        }
                    }
                }
            }
        }
    }

    /// Advances to the next week of the season.
    pub fn advance_week(&mut self) {
        if self.current_week <= self.total_weeks {
            self.current_week += 1;
        }
    }

    /// Returns true if the season is complete (all weeks have been played).
    pub fn is_complete(&self) -> bool {
        self.current_week > self.total_weeks
    }

    /// Returns the standings for a specific conference.
    pub fn conference_standings(&self, conference: &str) -> Vec<(Team, TeamRecord)> {
        let mut standings: Vec<(Team, TeamRecord)> = self
            .teams
            .iter()
            .filter(|t| t.conference == conference)
            .filter_map(|t| self.team_records.get(&t.id).map(|r| (t.clone(), *r)))
            .collect();

        // Sort by conference winning percentage (descending)
        standings.sort_by(|a, b| {
            b.1.conference_winning_percentage()
                .partial_cmp(&a.1.conference_winning_percentage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        standings
    }

    /// Returns the standings for a specific division within a conference.
    pub fn division_standings(&self, conference: &str, division: &str) -> Vec<(Team, TeamRecord)> {
        let mut standings: Vec<(Team, TeamRecord)> = self
            .teams
            .iter()
            .filter(|t| t.conference == conference && t.division.as_deref() == Some(division))
            .filter_map(|t| self.team_records.get(&t.id).map(|r| (t.clone(), *r)))
            .collect();

        // Sort by conference winning percentage (descending)
        standings.sort_by(|a, b| {
            b.1.conference_winning_percentage()
                .partial_cmp(&a.1.conference_winning_percentage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        standings
    }
}

/// Builder for creating Season instances with a fluent API.
#[derive(Default)]
pub struct SeasonBuilder {
    year: Option<u16>,
    teams: Vec<Team>,
    total_weeks: Option<u8>,
}

impl SeasonBuilder {
    /// Creates a new SeasonBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the year of the season.
    pub fn year(mut self, year: u16) -> Self {
        self.year = Some(year);
        self
    }

    /// Adds a team to the season.
    pub fn add_team(mut self, team: Team) -> Self {
        self.teams.push(team);
        self
    }

    /// Adds multiple teams to the season.
    pub fn add_teams(mut self, teams: Vec<Team>) -> Self {
        self.teams.extend(teams);
        self
    }

    /// Sets the total number of weeks in the regular season.
    pub fn total_weeks(mut self, weeks: u8) -> Self {
        self.total_weeks = Some(weeks);
        self
    }

    /// Builds a Season from the builder.
    ///
    /// # Panics
    ///
    /// Panics if any required field is not set.
    pub fn build(self) -> Season {
        Season::new(
            self.year.expect("Season year is required"),
            self.teams,
            self.total_weeks.expect("Total weeks is required"),
        )
    }
}
