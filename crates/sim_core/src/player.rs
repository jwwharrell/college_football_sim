//! Stable player identity, profile, attributes, and eligibility state.

use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};

const MAX_SEASONS: u8 = 4;

/// Stable player identifier that survives roster and season transitions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(String);

impl PlayerId {
    pub fn new(value: impl Into<String>) -> SimResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SimError::InvalidParameter(
                "player id cannot be empty".into(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Common primary roster positions. Exact alignments belong to depth charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Quarterback,
    RunningBack,
    Fullback,
    WideReceiver,
    TightEnd,
    OffensiveLine,
    DefensiveLine,
    Edge,
    Linebacker,
    Cornerback,
    Safety,
    Kicker,
    Punter,
    LongSnapper,
    Athlete,
}

/// Academic/roster class, capped at the final modeled year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassYear {
    Freshman,
    Sophomore,
    Junior,
    Senior,
}

impl ClassYear {
    fn advanced(self) -> Self {
        match self {
            Self::Freshman => Self::Sophomore,
            Self::Sophomore => Self::Junior,
            Self::Junior | Self::Senior => Self::Senior,
        }
    }
}

/// Foundational, position-independent football attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAttributes {
    pub speed: u8,
    pub strength: u8,
    pub agility: u8,
    pub awareness: u8,
    pub stamina: u8,
}

impl PlayerAttributes {
    pub fn new(
        speed: u8,
        strength: u8,
        agility: u8,
        awareness: u8,
        stamina: u8,
    ) -> SimResult<Self> {
        let attributes = Self {
            speed,
            strength,
            agility,
            awareness,
            stamina,
        };
        attributes.validate()?;
        Ok(attributes)
    }

    pub fn validate(&self) -> SimResult<()> {
        for (name, value) in [
            ("speed", self.speed),
            ("strength", self.strength),
            ("agility", self.agility),
            ("awareness", self.awareness),
            ("stamina", self.stamina),
        ] {
            if value > 100 {
                return Err(SimError::InvalidParameter(format!(
                    "{name} attribute must be between 0 and 100"
                )));
            }
        }
        Ok(())
    }
}

/// Explicit four-season eligibility state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eligibility {
    pub class_year: ClassYear,
    pub seasons_played: u8,
    pub seasons_remaining: u8,
    pub redshirt_used: bool,
}

impl Eligibility {
    pub fn new(
        class_year: ClassYear,
        seasons_played: u8,
        seasons_remaining: u8,
        redshirt_used: bool,
    ) -> SimResult<Self> {
        let eligibility = Self {
            class_year,
            seasons_played,
            seasons_remaining,
            redshirt_used,
        };
        eligibility.validate()?;
        Ok(eligibility)
    }

    pub fn validate(&self) -> SimResult<()> {
        if self.seasons_played > MAX_SEASONS || self.seasons_remaining > MAX_SEASONS {
            return Err(SimError::InvalidParameter(
                "eligibility seasons must be between 0 and 4".into(),
            ));
        }
        if self.seasons_played + self.seasons_remaining > MAX_SEASONS {
            return Err(SimError::InvalidParameter(
                "eligibility seasons played plus remaining cannot exceed 4".into(),
            ));
        }
        Ok(())
    }

    pub fn can_return(&self) -> bool {
        self.seasons_remaining > 0
    }

    pub fn advance(&self, participation: SeasonParticipation) -> SimResult<Self> {
        if !self.can_return() {
            return Err(SimError::InvalidParameter(
                "player eligibility is exhausted".into(),
            ));
        }
        let mut next = self.clone();
        next.class_year = next.class_year.advanced();
        match participation {
            SeasonParticipation::SeasonUsed => {
                next.seasons_played += 1;
                next.seasons_remaining -= 1;
            }
            SeasonParticipation::Redshirted => {
                if next.redshirt_used {
                    return Err(SimError::InvalidParameter(
                        "player redshirt was already used".into(),
                    ));
                }
                next.redshirt_used = true;
            }
        }
        next.validate()?;
        Ok(next)
    }
}

/// Declared participation outcome used to advance eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonParticipation {
    SeasonUsed,
    Redshirted,
}

/// A validated player profile with stable identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    id: PlayerId,
    first_name: String,
    last_name: String,
    position: Position,
    attributes: PlayerAttributes,
    eligibility: Eligibility,
}

impl Player {
    pub fn new(
        id: impl Into<String>,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        position: Position,
        attributes: PlayerAttributes,
        eligibility: Eligibility,
    ) -> SimResult<Self> {
        let player = Self {
            id: PlayerId::new(id)?,
            first_name: first_name.into(),
            last_name: last_name.into(),
            position,
            attributes,
            eligibility,
        };
        player.validate()?;
        Ok(player)
    }

    pub fn validate(&self) -> SimResult<()> {
        PlayerId::new(self.id.0.clone())?;
        for (field, value) in [
            ("first name", self.first_name.as_str()),
            ("last name", self.last_name.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(SimError::InvalidParameter(format!(
                    "{field} cannot be empty"
                )));
            }
        }
        self.attributes.validate()?;
        self.eligibility.validate()
    }

    pub fn id(&self) -> &PlayerId {
        &self.id
    }
    pub fn first_name(&self) -> &str {
        &self.first_name
    }
    pub fn last_name(&self) -> &str {
        &self.last_name
    }
    pub fn position(&self) -> Position {
        self.position
    }
    pub fn attributes(&self) -> &PlayerAttributes {
        &self.attributes
    }
    pub fn eligibility(&self) -> &Eligibility {
        &self.eligibility
    }

    pub fn advance(&self, participation: SeasonParticipation) -> SimResult<Self> {
        let mut next = self.clone();
        next.eligibility = self.eligibility.advance(participation)?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attributes() -> PlayerAttributes {
        PlayerAttributes::new(80, 81, 82, 83, 84).unwrap()
    }
    fn eligibility() -> Eligibility {
        Eligibility::new(ClassYear::Freshman, 0, 4, false).unwrap()
    }
    fn player() -> Player {
        Player::new(
            "p-1",
            "Ada",
            "Lovelace",
            Position::Quarterback,
            attributes(),
            eligibility(),
        )
        .unwrap()
    }

    #[test]
    fn constructs_valid_player_and_preserves_rating_boundaries() {
        let p = player();
        assert_eq!(p.id().as_str(), "p-1");
        assert_eq!(
            PlayerAttributes::new(0, 100, 0, 100, 0).unwrap().strength,
            100
        );
    }

    #[test]
    fn rejects_each_empty_identity_field_and_each_excessive_rating() {
        for (id, first, last) in [(" ", "Ada", "L"), ("p", " ", "L"), ("p", "Ada", " ")] {
            assert!(Player::new(
                id,
                first,
                last,
                Position::Athlete,
                attributes(),
                eligibility()
            )
            .is_err());
        }
        for values in [
            [101, 0, 0, 0, 0],
            [0, 101, 0, 0, 0],
            [0, 0, 101, 0, 0],
            [0, 0, 0, 101, 0],
            [0, 0, 0, 0, 101],
        ] {
            assert!(
                PlayerAttributes::new(values[0], values[1], values[2], values[3], values[4])
                    .is_err()
            );
        }
    }

    #[test]
    fn validates_eligibility_bounds_and_totals() {
        assert!(Eligibility::new(ClassYear::Freshman, 4, 0, false).is_ok());
        assert!(Eligibility::new(ClassYear::Freshman, 5, 0, false).is_err());
        assert!(Eligibility::new(ClassYear::Freshman, 0, 5, false).is_err());
        assert!(Eligibility::new(ClassYear::Freshman, 2, 3, false).is_err());
        assert!(!Eligibility::new(ClassYear::Senior, 4, 0, true)
            .unwrap()
            .can_return());
    }

    #[test]
    fn advances_season_and_redshirt_without_mutating_source() {
        let p = player();
        let used = p.advance(SeasonParticipation::SeasonUsed).unwrap();
        assert_eq!(used.id(), p.id());
        assert_eq!(used.eligibility().class_year, ClassYear::Sophomore);
        assert_eq!(
            (
                used.eligibility().seasons_played,
                used.eligibility().seasons_remaining
            ),
            (1, 3)
        );
        assert_eq!(p.eligibility().class_year, ClassYear::Freshman);

        let redshirted = p.advance(SeasonParticipation::Redshirted).unwrap();
        assert!(redshirted.eligibility().redshirt_used);
        assert_eq!(
            (
                redshirted.eligibility().seasons_played,
                redshirted.eligibility().seasons_remaining
            ),
            (0, 4)
        );
        assert!(redshirted.advance(SeasonParticipation::Redshirted).is_err());
        assert!(redshirted.eligibility().redshirt_used);
    }

    #[test]
    fn exhausted_player_cannot_advance() {
        let exhausted = Eligibility::new(ClassYear::Senior, 4, 0, true).unwrap();
        assert!(exhausted.advance(SeasonParticipation::SeasonUsed).is_err());
    }

    #[test]
    fn player_values_round_trip_with_stable_snake_case_enums() {
        let p = player();
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"quarterback\""));
        assert!(json.contains("\"freshman\""));
        assert_eq!(serde_json::from_str::<Player>(&json).unwrap(), p);
        assert_eq!(
            serde_json::to_string(&SeasonParticipation::SeasonUsed).unwrap(),
            "\"season_used\""
        );
        assert_eq!(
            serde_json::to_string(&Position::LongSnapper).unwrap(),
            "\"long_snapper\""
        );
        assert_eq!(
            serde_json::from_str::<PlayerAttributes>(
                &serde_json::to_string(p.attributes()).unwrap()
            )
            .unwrap(),
            *p.attributes()
        );
        assert_eq!(
            serde_json::from_str::<Eligibility>(&serde_json::to_string(p.eligibility()).unwrap())
                .unwrap(),
            *p.eligibility()
        );
    }
}
