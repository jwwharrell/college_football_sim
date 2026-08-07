//! Manager-controlled positional depth charts and deterministic unit strengths.

use crate::player::{PlayerAttributes, PlayerId, Position};
use crate::roster::{Roster, TeamId};
use crate::simulation::MatchupModifiers;
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const STRENGTH_FORMULA_VERSION: &str = "foundational-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Offense,
    Defense,
    SpecialTeams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotRole {
    Quarterback,
    RunningBack,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotOrdinal(u8);
impl SlotOrdinal {
    pub fn new(value: u8) -> SimResult<Self> {
        if value == 0 {
            Err(invalid("slot ordinal must be positive"))
        } else {
            Ok(Self(value))
        }
    }
    pub fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DepthChartSlot {
    pub unit: Unit,
    pub role: SlotRole,
    pub ordinal: SlotOrdinal,
}
impl DepthChartSlot {
    pub fn new(unit: Unit, role: SlotRole, ordinal: u8) -> SimResult<Self> {
        let slot = Self {
            unit,
            role,
            ordinal: SlotOrdinal::new(ordinal)?,
        };
        if canonical_slots().contains(&slot) {
            Ok(slot)
        } else {
            Err(invalid(format!("non-canonical depth-chart slot: {slot:?}")))
        }
    }
    pub fn accepts(self, position: Position) -> bool {
        let expected = match self.role {
            SlotRole::Quarterback => Position::Quarterback,
            SlotRole::RunningBack => Position::RunningBack,
            SlotRole::WideReceiver => Position::WideReceiver,
            SlotRole::TightEnd => Position::TightEnd,
            SlotRole::OffensiveLine => Position::OffensiveLine,
            SlotRole::DefensiveLine => Position::DefensiveLine,
            SlotRole::Edge => Position::Edge,
            SlotRole::Linebacker => Position::Linebacker,
            SlotRole::Cornerback => Position::Cornerback,
            SlotRole::Safety => Position::Safety,
            SlotRole::Kicker => Position::Kicker,
            SlotRole::Punter => Position::Punter,
            SlotRole::LongSnapper => Position::LongSnapper,
        };
        position == expected || (position == Position::Athlete && self.unit != Unit::SpecialTeams)
    }
}

pub fn canonical_slots() -> Vec<DepthChartSlot> {
    let mut slots = Vec::new();
    let mut add = |unit, role, count| {
        for ordinal in 1..=count {
            slots.push(DepthChartSlot {
                unit,
                role,
                ordinal: SlotOrdinal(ordinal),
            });
        }
    };
    add(Unit::Offense, SlotRole::Quarterback, 1);
    add(Unit::Offense, SlotRole::RunningBack, 1);
    add(Unit::Offense, SlotRole::WideReceiver, 3);
    add(Unit::Offense, SlotRole::TightEnd, 1);
    add(Unit::Offense, SlotRole::OffensiveLine, 5);
    add(Unit::Defense, SlotRole::DefensiveLine, 4);
    add(Unit::Defense, SlotRole::Edge, 2);
    add(Unit::Defense, SlotRole::Linebacker, 2);
    add(Unit::Defense, SlotRole::Cornerback, 2);
    add(Unit::Defense, SlotRole::Safety, 1);
    add(Unit::SpecialTeams, SlotRole::Kicker, 1);
    add(Unit::SpecialTeams, SlotRole::Punter, 1);
    add(Unit::SpecialTeams, SlotRole::LongSnapper, 1);
    slots
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthAssignment {
    pub slot: DepthChartSlot,
    pub players: Vec<PlayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthChart {
    team_id: TeamId,
    season_year: u16,
    assignments: Vec<DepthAssignment>,
}
impl DepthChart {
    pub fn new(roster: &Roster, assignments: Vec<DepthAssignment>) -> SimResult<Self> {
        let mut by_slot = BTreeMap::new();
        for assignment in assignments {
            if !canonical_slots().contains(&assignment.slot) {
                return Err(invalid(format!(
                    "non-canonical slot: {:?}",
                    assignment.slot
                )));
            }
            if assignment.players.is_empty() {
                return Err(invalid(format!(
                    "missing starter for slot: {:?}",
                    assignment.slot
                )));
            }
            if by_slot
                .insert(assignment.slot, assignment.players)
                .is_some()
            {
                return Err(invalid(format!("duplicate slot: {:?}", assignment.slot)));
            }
        }
        for slot in canonical_slots() {
            if !by_slot.contains_key(&slot) {
                return Err(invalid(format!("missing required slot: {slot:?}")));
            }
        }
        let mut seen: BTreeMap<Unit, BTreeSet<PlayerId>> = BTreeMap::new();
        for (slot, ids) in &by_slot {
            for id in ids {
                let player = roster.get(id).ok_or_else(|| {
                    invalid(format!(
                        "player {} not on roster for slot {slot:?}",
                        id.as_str()
                    ))
                })?;
                if !player.eligibility().can_return() {
                    return Err(invalid(format!(
                        "player {} has exhausted eligibility",
                        id.as_str()
                    )));
                }
                if !slot.accepts(player.position()) {
                    return Err(invalid(format!(
                        "player {} is incompatible with slot {slot:?}",
                        id.as_str()
                    )));
                }
                if !seen.entry(slot.unit).or_default().insert(id.clone()) {
                    return Err(invalid(format!(
                        "player {} is assigned more than once in {:?}",
                        id.as_str(),
                        slot.unit
                    )));
                }
            }
        }
        Ok(Self {
            team_id: roster.team_id().clone(),
            season_year: roster.season_year(),
            assignments: by_slot
                .into_iter()
                .map(|(slot, players)| DepthAssignment { slot, players })
                .collect(),
        })
    }
    pub fn team_id(&self) -> &TeamId {
        &self.team_id
    }
    pub fn season_year(&self) -> u16 {
        self.season_year
    }
    pub fn assignments(&self) -> impl ExactSizeIterator<Item = &DepthAssignment> {
        self.assignments.iter()
    }
    pub fn validate(&self, roster: &Roster) -> SimResult<()> {
        self.ensure_roster(roster)?;
        Self::new(roster, self.assignments.clone()).map(|_| ())
    }
    pub fn players_at(&self, slot: DepthChartSlot) -> Option<&[PlayerId]> {
        self.assignments
            .binary_search_by_key(&slot, |a| a.slot)
            .ok()
            .map(|i| self.assignments[i].players.as_slice())
    }
    pub fn replace_slot(
        &mut self,
        roster: &Roster,
        slot: DepthChartSlot,
        players: Vec<PlayerId>,
    ) -> SimResult<()> {
        self.ensure_roster(roster)?;
        let mut candidate = self.assignments.clone();
        let index = candidate
            .binary_search_by_key(&slot, |a| a.slot)
            .map_err(|_| invalid(format!("unknown slot: {slot:?}")))?;
        candidate[index].players = players;
        *self = Self::new(roster, candidate)?;
        Ok(())
    }
    pub fn remove_player(
        &mut self,
        roster: &Roster,
        slot: DepthChartSlot,
        id: &PlayerId,
    ) -> SimResult<()> {
        let mut players = self
            .players_at(slot)
            .ok_or_else(|| invalid(format!("unknown slot: {slot:?}")))?
            .to_vec();
        let index = players
            .iter()
            .position(|candidate| candidate == id)
            .ok_or_else(|| invalid(format!("player {} is not assigned to slot", id.as_str())))?;
        players.remove(index);
        self.replace_slot(roster, slot, players)
    }
    fn ensure_roster(&self, roster: &Roster) -> SimResult<()> {
        if &self.team_id != roster.team_id() || self.season_year != roster.season_year() {
            Err(invalid("depth chart and roster team/season do not match"))
        } else {
            Ok(())
        }
    }
    pub fn strengths(&self, roster: &Roster) -> SimResult<UnitStrengths> {
        self.ensure_roster(roster)?;
        let mut totals: BTreeMap<Unit, (u64, u64)> = BTreeMap::new();
        for assignment in &self.assignments {
            for (depth, id) in assignment.players.iter().enumerate() {
                let player = roster
                    .get(id)
                    .ok_or_else(|| invalid(format!("player {} not on roster", id.as_str())))?;
                let depth_weight = match depth {
                    0 => 100_u64,
                    1 => 50,
                    2 => 25,
                    _ => 10,
                };
                let entry = totals.entry(assignment.slot.unit).or_default();
                entry.0 += u64::from(weighted_attributes(
                    assignment.slot.role,
                    player.attributes(),
                )) * depth_weight;
                entry.1 += depth_weight;
            }
        }
        let value = |unit| {
            let (sum, weight) = totals[&unit];
            ((sum + weight / 2) / weight).min(100) as u8
        };
        Ok(UnitStrengths {
            offense: value(Unit::Offense),
            defense: value(Unit::Defense),
            special_teams: value(Unit::SpecialTeams),
        })
    }
}

/// Attribute weights total 100; starters/backups receive depth weights 100/50/25/10.
fn weighted_attributes(role: SlotRole, a: &PlayerAttributes) -> u32 {
    let weights = match role {
        SlotRole::Quarterback => [10, 5, 10, 55, 20],
        SlotRole::RunningBack
        | SlotRole::WideReceiver
        | SlotRole::Cornerback
        | SlotRole::Safety => [30, 10, 25, 20, 15],
        SlotRole::OffensiveLine | SlotRole::DefensiveLine | SlotRole::LongSnapper => {
            [5, 40, 10, 30, 15]
        }
        SlotRole::TightEnd | SlotRole::Edge | SlotRole::Linebacker => [20, 25, 20, 20, 15],
        SlotRole::Kicker | SlotRole::Punter => [5, 10, 10, 55, 20],
    };
    [a.speed, a.strength, a.agility, a.awareness, a.stamina]
        .into_iter()
        .zip(weights)
        .map(|(v, w)| u32::from(v) * w)
        .sum::<u32>()
        / 100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitStrengths {
    pub offense: u8,
    pub defense: u8,
    pub special_teams: u8,
}
impl UnitStrengths {
    fn modifier(value: u8) -> f64 {
        (f64::from(value.clamp(0, 100)) - 50.0) / 2.0
    }
    pub fn as_home_modifiers(self) -> MatchupModifiers {
        MatchupModifiers {
            home_offense: Self::modifier(self.offense),
            home_defense: Self::modifier(self.defense),
            home_special_teams: Self::modifier(self.special_teams),
            ..Default::default()
        }
    }
    pub fn as_away_modifiers(self) -> MatchupModifiers {
        MatchupModifiers {
            away_offense: Self::modifier(self.offense),
            away_defense: Self::modifier(self.defense),
            away_special_teams: Self::modifier(self.special_teams),
            ..Default::default()
        }
    }

    /// Composes both teams' strengths into one complete matchup modifier value.
    pub fn matchup_modifiers(home: Self, away: Self) -> MatchupModifiers {
        MatchupModifiers {
            home_offense: Self::modifier(home.offense),
            away_offense: Self::modifier(away.offense),
            home_defense: Self::modifier(home.defense),
            away_defense: Self::modifier(away.defense),
            home_special_teams: Self::modifier(home.special_teams),
            away_special_teams: Self::modifier(away.special_teams),
        }
    }
}

fn invalid(message: impl Into<String>) -> SimError {
    SimError::InvalidParameter(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{ClassYear, Eligibility, Player};

    fn position(role: SlotRole) -> Position {
        match role {
            SlotRole::Quarterback => Position::Quarterback,
            SlotRole::RunningBack => Position::RunningBack,
            SlotRole::WideReceiver => Position::WideReceiver,
            SlotRole::TightEnd => Position::TightEnd,
            SlotRole::OffensiveLine => Position::OffensiveLine,
            SlotRole::DefensiveLine => Position::DefensiveLine,
            SlotRole::Edge => Position::Edge,
            SlotRole::Linebacker => Position::Linebacker,
            SlotRole::Cornerback => Position::Cornerback,
            SlotRole::Safety => Position::Safety,
            SlotRole::Kicker => Position::Kicker,
            SlotRole::Punter => Position::Punter,
            SlotRole::LongSnapper => Position::LongSnapper,
        }
    }
    fn id(slot: DepthChartSlot) -> String {
        format!("{:?}-{}", slot.role, slot.ordinal.get())
    }
    fn player(id: &str, position: Position, rating: u8, remaining: u8) -> Player {
        Player::new(
            id,
            "Depth",
            "Player",
            position,
            PlayerAttributes::new(rating, rating, rating, rating, rating).unwrap(),
            Eligibility::new(ClassYear::Junior, 4 - remaining, remaining, false).unwrap(),
        )
        .unwrap()
    }
    fn fixture() -> (Roster, Vec<DepthAssignment>) {
        let slots = canonical_slots();
        let mut players: Vec<_> = slots
            .iter()
            .map(|slot| player(&id(*slot), position(slot.role), 50, 2))
            .collect();
        players.push(player("athlete", Position::Athlete, 60, 2));
        players.push(player("strong-qb", Position::Quarterback, 100, 2));
        players.push(player("exhausted-qb", Position::Quarterback, 50, 0));
        let assignments = slots
            .into_iter()
            .map(|slot| DepthAssignment {
                slot,
                players: vec![PlayerId::new(id(slot)).unwrap()],
            })
            .collect();
        (Roster::new("team", 2026, players).unwrap(), assignments)
    }
    fn slot(unit: Unit, role: SlotRole, ordinal: u8) -> DepthChartSlot {
        DepthChartSlot::new(unit, role, ordinal).unwrap()
    }

    #[test]
    fn complete_chart_is_canonical_and_missing_slot_fails() {
        let (roster, assignments) = fixture();
        let mut reversed = assignments.clone();
        reversed.reverse();
        let chart = DepthChart::new(&roster, reversed).unwrap();
        assert_eq!(
            chart.assignments().map(|a| a.slot).collect::<Vec<_>>(),
            canonical_slots()
        );
        assert!(DepthChart::new(&roster, assignments[..assignments.len() - 1].to_vec()).is_err());
    }

    #[test]
    fn validates_membership_eligibility_compatibility_and_specialists() {
        let (roster, assignments) = fixture();
        let qb = slot(Unit::Offense, SlotRole::Quarterback, 1);
        let kicker = slot(Unit::SpecialTeams, SlotRole::Kicker, 1);
        for (target, candidate) in [
            (qb, "missing"),
            (qb, "exhausted-qb"),
            (qb, "Kicker-1"),
            (kicker, "athlete"),
        ] {
            let mut changed = assignments.clone();
            changed
                .iter_mut()
                .find(|a| a.slot == target)
                .unwrap()
                .players = vec![PlayerId::new(candidate).unwrap()];
            assert!(DepthChart::new(&roster, changed).is_err());
        }
    }

    #[test]
    fn edits_are_atomic_and_cross_unit_overlap_is_allowed() {
        let (roster, assignments) = fixture();
        let mut chart = DepthChart::new(&roster, assignments).unwrap();
        let qb = slot(Unit::Offense, SlotRole::Quarterback, 1);
        let rb = slot(Unit::Offense, SlotRole::RunningBack, 1);
        let before = chart.clone();
        assert!(chart
            .replace_slot(&roster, rb, chart.players_at(qb).unwrap().to_vec())
            .is_err());
        assert_eq!(chart, before);
        chart
            .replace_slot(&roster, qb, vec![PlayerId::new("athlete").unwrap()])
            .unwrap();
        assert_eq!(chart.players_at(qb).unwrap()[0].as_str(), "athlete");
        let before = chart.clone();
        assert!(chart
            .remove_player(&roster, qb, &PlayerId::new("athlete").unwrap())
            .is_err());
        assert_eq!(chart, before);
        let other_roster = Roster::new("other", 2026, roster.players().cloned().collect()).unwrap();
        assert!(chart.validate(&other_roster).is_err());
    }

    #[test]
    fn strength_is_bounded_reproducible_and_depth_weighted() {
        let (roster, assignments) = fixture();
        let mut chart = DepthChart::new(&roster, assignments).unwrap();
        let base = chart.strengths(&roster).unwrap();
        assert_eq!(base, chart.strengths(&roster).unwrap());
        assert_eq!(
            base,
            UnitStrengths {
                offense: 50,
                defense: 50,
                special_teams: 50
            }
        );
        let qb = slot(Unit::Offense, SlotRole::Quarterback, 1);
        let normal = chart.players_at(qb).unwrap()[0].clone();
        chart
            .replace_slot(
                &roster,
                qb,
                vec![PlayerId::new("strong-qb").unwrap(), normal.clone()],
            )
            .unwrap();
        let strong_first = chart.strengths(&roster).unwrap().offense;
        chart
            .replace_slot(
                &roster,
                qb,
                vec![normal, PlayerId::new("strong-qb").unwrap()],
            )
            .unwrap();
        assert!(strong_first >= chart.strengths(&roster).unwrap().offense);
        assert!(strong_first <= 100);
    }

    #[test]
    fn serde_is_stable_and_modifiers_are_neutral_and_bounded() {
        let (roster, assignments) = fixture();
        let chart = DepthChart::new(&roster, assignments).unwrap();
        let json = serde_json::to_string(&chart).unwrap();
        assert!(json.contains("\"special_teams\""));
        let decoded: DepthChart = serde_json::from_str(&json).unwrap();
        decoded.validate(&roster).unwrap();
        assert_eq!(decoded, chart);
        assert_eq!(json, serde_json::to_string(&decoded).unwrap());
        assert_eq!(
            UnitStrengths {
                offense: 50,
                defense: 50,
                special_teams: 50
            }
            .as_home_modifiers(),
            MatchupModifiers::default()
        );
        let high = UnitStrengths {
            offense: 100,
            defense: 100,
            special_teams: 100,
        }
        .as_away_modifiers();
        assert_eq!(
            (
                high.away_offense,
                high.away_defense,
                high.away_special_teams
            ),
            (25.0, 25.0, 25.0)
        );
    }
}
