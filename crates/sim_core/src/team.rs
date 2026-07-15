//! Team module for the college football simulator.
//!
//! This module defines the Team type and related functionality.

use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a college football team with its attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Team {
    /// Unique identifier for the team
    pub id: String,
    /// Full name of the team (e.g., "Alabama Crimson Tide")
    pub name: String,
    /// Short name or abbreviation (e.g., "ALA")
    pub abbreviation: String,
    /// Team mascot (e.g., "Crimson Tide")
    pub mascot: String,
    /// Conference the team belongs to
    pub conference: String,
    /// Division within the conference (if applicable)
    pub division: Option<String>,
    /// Team's home location
    pub location: String,
    /// Overall team rating (0-100)
    pub rating: u8,
    /// Offensive rating (0-100)
    pub offense_rating: u8,
    /// Defensive rating (0-100)
    pub defense_rating: u8,
    /// Special teams rating (0-100)
    pub special_teams_rating: u8,
}

impl Team {
    /// Creates a new team with the given attributes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        abbreviation: impl Into<String>,
        mascot: impl Into<String>,
        conference: impl Into<String>,
        division: Option<String>,
        location: impl Into<String>,
        rating: u8,
        offense_rating: u8,
        defense_rating: u8,
        special_teams_rating: u8,
    ) -> SimResult<Self> {
        let team = Self {
            id: id.into(),
            name: name.into(),
            abbreviation: abbreviation.into(),
            mascot: mascot.into(),
            conference: conference.into(),
            division,
            location: location.into(),
            rating,
            offense_rating,
            defense_rating,
            special_teams_rating,
        };
        team.validate()?;
        Ok(team)
    }

    /// Validates required text fields and all 0-100 ratings.
    pub fn validate(&self) -> SimResult<()> {
        for (name, value) in [
            ("team id", self.id.as_str()),
            ("team name", self.name.as_str()),
            ("abbreviation", self.abbreviation.as_str()),
            ("mascot", self.mascot.as_str()),
            ("conference", self.conference.as_str()),
            ("location", self.location.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(SimError::InvalidParameter(format!(
                    "{name} cannot be empty"
                )));
            }
        }
        for (name, rating) in [
            ("overall", self.rating),
            ("offense", self.offense_rating),
            ("defense", self.defense_rating),
            ("special teams", self.special_teams_rating),
        ] {
            if rating > 100 {
                return Err(SimError::InvalidParameter(format!(
                    "{name} rating must be between 0 and 100"
                )));
            }
        }
        Ok(())
    }

    /// Returns the full name of the team.
    pub fn full_name(&self) -> String {
        format!("{} {}", self.location, self.mascot)
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.abbreviation)
    }
}

/// Builder for creating Team instances with a fluent API.
#[derive(Default)]
pub struct TeamBuilder {
    id: Option<String>,
    name: Option<String>,
    abbreviation: Option<String>,
    mascot: Option<String>,
    conference: Option<String>,
    division: Option<String>,
    location: Option<String>,
    rating: Option<u8>,
    offense_rating: Option<u8>,
    defense_rating: Option<u8>,
    special_teams_rating: Option<u8>,
}

impl TeamBuilder {
    /// Creates a new TeamBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the team ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the team name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the team abbreviation.
    pub fn abbreviation(mut self, abbreviation: impl Into<String>) -> Self {
        self.abbreviation = Some(abbreviation.into());
        self
    }

    /// Sets the team mascot.
    pub fn mascot(mut self, mascot: impl Into<String>) -> Self {
        self.mascot = Some(mascot.into());
        self
    }

    /// Sets the team conference.
    pub fn conference(mut self, conference: impl Into<String>) -> Self {
        self.conference = Some(conference.into());
        self
    }

    /// Sets the team division.
    pub fn division(mut self, division: impl Into<String>) -> Self {
        self.division = Some(division.into());
        self
    }

    /// Sets the team location.
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Sets the overall team rating.
    pub fn rating(mut self, rating: u8) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Sets the offensive rating.
    pub fn offense_rating(mut self, rating: u8) -> Self {
        self.offense_rating = Some(rating);
        self
    }

    /// Sets the defensive rating.
    pub fn defense_rating(mut self, rating: u8) -> Self {
        self.defense_rating = Some(rating);
        self
    }

    /// Sets the special teams rating.
    pub fn special_teams_rating(mut self, rating: u8) -> Self {
        self.special_teams_rating = Some(rating);
        self
    }

    /// Builds a Team from the builder.
    ///
    /// # Panics
    ///
    /// Panics if any required field is not set.
    pub fn build(self) -> SimResult<Team> {
        Team::new(
            self.id
                .ok_or_else(|| SimError::InvalidParameter("team id is required".into()))?,
            self.name
                .ok_or_else(|| SimError::InvalidParameter("team name is required".into()))?,
            self.abbreviation
                .ok_or_else(|| SimError::InvalidParameter("abbreviation is required".into()))?,
            self.mascot
                .ok_or_else(|| SimError::InvalidParameter("mascot is required".into()))?,
            self.conference
                .ok_or_else(|| SimError::InvalidParameter("conference is required".into()))?,
            self.division,
            self.location
                .ok_or_else(|| SimError::InvalidParameter("location is required".into()))?,
            self.rating
                .ok_or_else(|| SimError::InvalidParameter("team rating is required".into()))?,
            self.offense_rating
                .ok_or_else(|| SimError::InvalidParameter("offense rating is required".into()))?,
            self.defense_rating
                .ok_or_else(|| SimError::InvalidParameter("defense rating is required".into()))?,
            self.special_teams_rating.ok_or_else(|| {
                SimError::InvalidParameter("special teams rating is required".into())
            })?,
        )
    }
}
