//! Season-scoped, deterministic program rosters.

use crate::player::{Player, PlayerId, SeasonParticipation};
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Validated program identifier used by a roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TeamId(String);

impl TeamId {
    pub fn new(value: impl Into<String>) -> SimResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SimError::InvalidParameter("team id cannot be empty".into()));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A program's validated roster for one season.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roster {
    team_id: TeamId,
    season_year: u16,
    /// Kept sorted by player ID so iteration and serde output are canonical.
    players: Vec<Player>,
}

impl Roster {
    pub fn new(
        team_id: impl Into<String>,
        season_year: u16,
        mut players: Vec<Player>,
    ) -> SimResult<Self> {
        let team_id = TeamId::new(team_id)?;
        if season_year == 0 {
            return Err(SimError::InvalidParameter(
                "season year must be positive".into(),
            ));
        }
        for player in &players {
            player.validate()?;
        }
        players.sort_by(|left, right| left.id().cmp(right.id()));
        for pair in players.windows(2) {
            if pair[0].id() == pair[1].id() {
                return Err(SimError::InvalidParameter(format!(
                    "duplicate player id: {}",
                    pair[0].id().as_str()
                )));
            }
        }
        Ok(Self {
            team_id,
            season_year,
            players,
        })
    }

    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
    pub fn season_year(&self) -> u16 {
        self.season_year
    }
    pub fn players(&self) -> impl ExactSizeIterator<Item = &Player> {
        self.players.iter()
    }
    pub fn len(&self) -> usize {
        self.players.len()
    }
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn get(&self, id: &PlayerId) -> Option<&Player> {
        self.players
            .binary_search_by(|player| player.id().cmp(id))
            .ok()
            .map(|index| &self.players[index])
    }

    pub fn add(&mut self, player: Player) -> SimResult<()> {
        player.validate()?;
        match self
            .players
            .binary_search_by(|current| current.id().cmp(player.id()))
        {
            Ok(_) => Err(SimError::InvalidParameter(format!(
                "duplicate player id: {}",
                player.id().as_str()
            ))),
            Err(index) => {
                self.players.insert(index, player);
                Ok(())
            }
        }
    }

    pub fn remove(&mut self, id: &PlayerId) -> SimResult<Player> {
        let index = self
            .players
            .binary_search_by(|player| player.id().cmp(id))
            .map_err(|_| {
                SimError::InvalidParameter(format!("player id not on roster: {}", id.as_str()))
            })?;
        Ok(self.players.remove(index))
    }

    /// Builds a new roster atomically; this roster is never mutated.
    pub fn transition_to_next_season(
        &self,
        outcomes: &BTreeMap<PlayerId, SeasonParticipation>,
    ) -> SimResult<RosterTransition> {
        let roster_ids: BTreeSet<_> = self.players.iter().map(|player| player.id()).collect();
        for id in outcomes.keys() {
            if !roster_ids.contains(id) {
                return Err(SimError::InvalidParameter(format!(
                    "participation outcome supplied for unknown player: {}",
                    id.as_str()
                )));
            }
        }

        let mut returning = Vec::with_capacity(self.players.len());
        let mut eligibility_departures = Vec::new();
        for player in &self.players {
            let outcome = outcomes.get(player.id()).ok_or_else(|| {
                SimError::InvalidParameter(format!(
                    "missing participation outcome for player: {}",
                    player.id().as_str()
                ))
            })?;
            let advanced = player.advance(*outcome)?;
            if advanced.eligibility().can_return() {
                returning.push(advanced);
            } else {
                eligibility_departures.push(advanced);
            }
        }

        let next_year = self.season_year.checked_add(1).ok_or_else(|| {
            SimError::InvalidParameter("season year cannot advance beyond u16 maximum".into())
        })?;
        Ok(RosterTransition {
            roster: Self::new(self.team_id.0.clone(), next_year, returning)?,
            eligibility_departures,
        })
    }
}

/// Result of advancing every member of a roster by one season.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterTransition {
    pub roster: Roster,
    /// Canonically ordered by stable player ID.
    pub eligibility_departures: Vec<Player>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{ClassYear, Eligibility, PlayerAttributes, Position};

    fn player(id: &str, remaining: u8) -> Player {
        Player::new(
            id,
            "Test",
            "Player",
            Position::Athlete,
            PlayerAttributes::new(50, 50, 50, 50, 50).unwrap(),
            Eligibility::new(ClassYear::Junior, 4 - remaining, remaining, false).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn rejects_invalid_roots_and_duplicate_construction_atomically() {
        assert!(Roster::new(" ", 2026, vec![]).is_err());
        assert!(Roster::new("team", 0, vec![]).is_err());
        assert!(Roster::new("team", 2026, vec![player("p", 2), player("p", 2)]).is_err());
    }

    #[test]
    fn orders_independently_of_insertion_and_supports_lookup_add_remove() {
        let mut roster = Roster::new("team", 2026, vec![player("b", 2), player("a", 2)]).unwrap();
        assert_eq!(
            roster
                .players()
                .map(|p| p.id().as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        let before = roster.clone();
        assert!(roster.add(player("a", 3)).is_err());
        assert_eq!(roster, before);
        roster.add(player("c", 2)).unwrap();
        let c = PlayerId::new("c").unwrap();
        assert!(roster.get(&c).is_some());
        assert_eq!(roster.remove(&c).unwrap().id(), &c);
        assert!(roster.get(&c).is_none());
    }

    #[test]
    fn transition_requires_every_outcome_and_preserves_source() {
        let roster = Roster::new("team", 2026, vec![player("a", 2), player("b", 1)]).unwrap();
        let source = roster.clone();
        let mut incomplete = BTreeMap::new();
        incomplete.insert(PlayerId::new("a").unwrap(), SeasonParticipation::SeasonUsed);
        assert!(roster.transition_to_next_season(&incomplete).is_err());
        assert_eq!(roster, source);

        incomplete.insert(PlayerId::new("b").unwrap(), SeasonParticipation::SeasonUsed);
        let transition = roster.transition_to_next_season(&incomplete).unwrap();
        assert_eq!(transition.roster.season_year(), 2027);
        assert_eq!(
            transition
                .roster
                .players()
                .map(|p| p.id().as_str())
                .collect::<Vec<_>>(),
            ["a"]
        );
        assert_eq!(transition.eligibility_departures[0].id().as_str(), "b");
        assert_eq!(
            transition
                .roster
                .get(&PlayerId::new("a").unwrap())
                .unwrap()
                .id()
                .as_str(),
            "a"
        );
        assert_eq!(roster, source);
    }

    #[test]
    fn invalid_transition_is_all_or_nothing() {
        let redshirt_used = Player::new(
            "b",
            "Test",
            "Player",
            Position::Athlete,
            PlayerAttributes::new(50, 50, 50, 50, 50).unwrap(),
            Eligibility::new(ClassYear::Junior, 2, 2, true).unwrap(),
        )
        .unwrap();
        let roster = Roster::new("team", 2026, vec![player("a", 2), redshirt_used]).unwrap();
        let mut outcomes = BTreeMap::new();
        outcomes.insert(PlayerId::new("a").unwrap(), SeasonParticipation::SeasonUsed);
        outcomes.insert(PlayerId::new("b").unwrap(), SeasonParticipation::Redshirted);
        assert!(roster.transition_to_next_season(&outcomes).is_err());
        assert_eq!(
            roster
                .get(&PlayerId::new("a").unwrap())
                .unwrap()
                .eligibility()
                .seasons_remaining,
            2
        );
    }

    #[test]
    fn json_round_trip_uses_canonical_player_order() {
        let first = Roster::new("team", 2026, vec![player("b", 2), player("a", 2)]).unwrap();
        let second = Roster::new("team", 2026, vec![player("a", 2), player("b", 2)]).unwrap();
        let first_json = serde_json::to_string(&first).unwrap();
        assert_eq!(first_json, serde_json::to_string(&second).unwrap());
        assert!(first_json.find("\"a\"").unwrap() < first_json.find("\"b\"").unwrap());
        assert_eq!(serde_json::from_str::<Roster>(&first_json).unwrap(), first);
    }
}
