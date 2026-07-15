//! Game module for the college football simulator.
//!
//! This module defines the Game type and related functionality.

use crate::team::Team;
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the status of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    /// Game is scheduled but not yet played
    Scheduled,
    /// Game is in progress
    InProgress,
    /// Game has been completed
    Completed,
    /// Game has been canceled
    Canceled,
}

/// Represents a quarter in a football game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quarter {
    First,
    Second,
    Third,
    Fourth,
    Overtime(u8), // Overtime number (1st OT, 2nd OT, etc.)
}

impl fmt::Display for Quarter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quarter::First => write!(f, "1st"),
            Quarter::Second => write!(f, "2nd"),
            Quarter::Third => write!(f, "3rd"),
            Quarter::Fourth => write!(f, "4th"),
            Quarter::Overtime(n) => write!(f, "OT{}", n),
        }
    }
}

/// Represents the score for a team in a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamScore {
    /// Total points scored
    pub total: u16,
    /// Points scored in the first quarter
    pub q1: u16,
    /// Points scored in the second quarter
    pub q2: u16,
    /// Points scored in the third quarter
    pub q3: u16,
    /// Points scored in the fourth quarter
    pub q4: u16,
    /// Points scored in overtime (all overtime periods combined)
    pub ot: u16,
}

impl TeamScore {
    /// Creates a new TeamScore with all values set to 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds points to the specified quarter.
    pub fn add_points(&mut self, quarter: Quarter, points: u16) {
        match quarter {
            Quarter::First => self.q1 += points,
            Quarter::Second => self.q2 += points,
            Quarter::Third => self.q3 += points,
            Quarter::Fourth => self.q4 += points,
            Quarter::Overtime(_) => self.ot += points,
        }
        self.total += points;
    }
}

/// Represents a college football game between two teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Unique identifier for the game
    pub id: String,
    /// Home team
    pub home_team: Team,
    /// Away team
    pub away_team: Team,
    /// Location of the game
    pub location: String,
    /// Week number in the season
    pub week: u8,
    /// Whether this is a conference game
    pub is_conference_game: bool,
    /// Whether this is a neutral site game
    pub is_neutral_site: bool,
    /// Current status of the game
    pub status: GameStatus,
    /// Current quarter (if in progress or completed)
    pub current_quarter: Option<Quarter>,
    /// Home team score
    pub home_score: TeamScore,
    /// Away team score
    pub away_score: TeamScore,
}

impl Game {
    /// Creates a new scheduled game between two teams.
    pub fn new(
        id: impl Into<String>,
        home_team: Team,
        away_team: Team,
        location: impl Into<String>,
        week: u8,
        is_conference_game: bool,
        is_neutral_site: bool,
    ) -> Self {
        Self {
            id: id.into(),
            home_team,
            away_team,
            location: location.into(),
            week,
            is_conference_game,
            is_neutral_site,
            status: GameStatus::Scheduled,
            current_quarter: None,
            home_score: TeamScore::new(),
            away_score: TeamScore::new(),
        }
    }

    /// Returns true if the game has been completed.
    pub fn is_completed(&self) -> bool {
        self.status == GameStatus::Completed
    }

    /// Returns the winning team, or None if the game is not completed or is a tie.
    pub fn winner(&self) -> Option<&Team> {
        if !self.is_completed() {
            return None;
        }

        match self.home_score.total.cmp(&self.away_score.total) {
            std::cmp::Ordering::Greater => Some(&self.home_team),
            std::cmp::Ordering::Less => Some(&self.away_team),
            std::cmp::Ordering::Equal => None, // Tie
        }
    }

    /// Returns the losing team, or None if the game is not completed or is a tie.
    pub fn loser(&self) -> Option<&Team> {
        if !self.is_completed() {
            return None;
        }

        match self.home_score.total.cmp(&self.away_score.total) {
            std::cmp::Ordering::Greater => Some(&self.away_team),
            std::cmp::Ordering::Less => Some(&self.home_team),
            std::cmp::Ordering::Equal => None, // Tie
        }
    }

    /// Returns true if the game ended in a tie.
    pub fn is_tie(&self) -> bool {
        self.is_completed() && self.home_score.total == self.away_score.total
    }

    /// Starts the game, changing its status to InProgress.
    pub fn start(&mut self) -> SimResult<()> {
        if self.status != GameStatus::Scheduled {
            return Err(SimError::InvalidGameStatus);
        }
        self.status = GameStatus::InProgress;
        self.current_quarter = Some(Quarter::First);
        Ok(())
    }

    /// Completes the game, changing its status to Completed.
    pub fn complete(&mut self) -> SimResult<()> {
        if self.status != GameStatus::InProgress {
            return Err(SimError::InvalidGameStatus);
        }
        self.status = GameStatus::Completed;
        Ok(())
    }

    /// Cancels the game, changing its status to Canceled.
    pub fn cancel(&mut self) -> SimResult<()> {
        if !matches!(self.status, GameStatus::Scheduled | GameStatus::InProgress) {
            return Err(SimError::InvalidGameStatus);
        }
        self.status = GameStatus::Canceled;
        self.current_quarter = None;
        Ok(())
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            GameStatus::Scheduled => {
                write!(
                    f,
                    "{} vs. {} (Week {}, Scheduled)",
                    self.away_team, self.home_team, self.week
                )
            }
            GameStatus::InProgress => {
                write!(
                    f,
                    "{} {} - {} {} ({:?})",
                    self.away_team,
                    self.away_score.total,
                    self.home_score.total,
                    self.home_team,
                    self.current_quarter.unwrap_or(Quarter::Fourth)
                )
            }
            GameStatus::Completed => {
                write!(
                    f,
                    "{} {} - {} {} (Final)",
                    self.away_team, self.away_score.total, self.home_score.total, self.home_team
                )
            }
            GameStatus::Canceled => {
                write!(f, "{} vs. {} (Canceled)", self.away_team, self.home_team)
            }
        }
    }
}

/// Builder for creating Game instances with a fluent API.
#[derive(Default)]
pub struct GameBuilder {
    id: Option<String>,
    home_team: Option<Team>,
    away_team: Option<Team>,
    location: Option<String>,
    week: Option<u8>,
    is_conference_game: bool,
    is_neutral_site: bool,
}

impl GameBuilder {
    /// Creates a new GameBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the game ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the home team.
    pub fn home_team(mut self, team: Team) -> Self {
        self.home_team = Some(team);
        self
    }

    /// Sets the away team.
    pub fn away_team(mut self, team: Team) -> Self {
        self.away_team = Some(team);
        self
    }

    /// Sets the game location.
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Sets the week number.
    pub fn week(mut self, week: u8) -> Self {
        self.week = Some(week);
        self
    }

    /// Sets whether this is a conference game.
    pub fn conference_game(mut self, is_conference_game: bool) -> Self {
        self.is_conference_game = is_conference_game;
        self
    }

    /// Sets whether this is a neutral site game.
    pub fn neutral_site(mut self, is_neutral_site: bool) -> Self {
        self.is_neutral_site = is_neutral_site;
        self
    }

    /// Builds a Game from the builder.
    ///
    /// # Panics
    ///
    /// Panics if any required field is not set.
    pub fn build(self) -> Game {
        let home_team = self.home_team.expect("Home team is required");
        let location = self.location.unwrap_or_else(|| home_team.location.clone());

        Game::new(
            self.id.expect("Game ID is required"),
            home_team,
            self.away_team.expect("Away team is required"),
            location,
            self.week.expect("Week number is required"),
            self.is_conference_game,
            self.is_neutral_site,
        )
    }
}
