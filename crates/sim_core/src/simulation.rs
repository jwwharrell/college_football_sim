//! Pure, deterministic possession-level game simulation.
//!
//! The public contract deliberately accepts team-level modifiers today. Future
//! roster units, schemes, coaches, fatigue, weather, and injuries can be folded
//! into [`MatchupModifiers`] without changing possession orchestration or result
//! consumers.

use crate::game::{Game, Quarter};
use crate::rng::SimRng;
use crate::team::Team;
use crate::{SimError, SimResult};
use serde::{Deserialize, Serialize};

/// Changes whenever stochastic mechanics or random draw ordering changes.
pub const ALGORITHM_VERSION: &str = "possession-v1";

/// The venue context for a simulated game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Venue {
    Home,
    Neutral,
}

/// Reserved extension point for dynasty-management effects.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct MatchupModifiers {
    /// Additive home offense rating adjustment.
    pub home_offense: f64,
    /// Additive away offense rating adjustment.
    pub away_offense: f64,
    /// Additive home defense rating adjustment.
    pub home_defense: f64,
    /// Additive away defense rating adjustment.
    pub away_defense: f64,
    /// Additive home special-teams rating adjustment.
    pub home_special_teams: f64,
    /// Additive away special-teams rating adjustment.
    pub away_special_teams: f64,
}

/// Immutable inputs for one game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matchup {
    pub game_id: String,
    pub home: Team,
    pub away: Team,
    pub location: String,
    pub week: u8,
    pub conference_game: bool,
    pub venue: Venue,
    pub modifiers: MatchupModifiers,
}

impl Matchup {
    pub fn validate(&self) -> SimResult<()> {
        if self.game_id.trim().is_empty() {
            return Err(invalid("game_id cannot be empty"));
        }
        if self.location.trim().is_empty() {
            return Err(invalid("location cannot be empty"));
        }
        if self.week == 0 {
            return Err(invalid("week must be greater than zero"));
        }
        if self.home.id == self.away.id {
            return Err(invalid("home and away team identifiers must differ"));
        }
        self.home.validate()?;
        self.away.validate()?;
        for (name, value) in [
            ("modifiers.home_offense", self.modifiers.home_offense),
            ("modifiers.away_offense", self.modifiers.away_offense),
            ("modifiers.home_defense", self.modifiers.home_defense),
            ("modifiers.away_defense", self.modifiers.away_defense),
            (
                "modifiers.home_special_teams",
                self.modifiers.home_special_teams,
            ),
            (
                "modifiers.away_special_teams",
                self.modifiers.away_special_teams,
            ),
        ] {
            finite(name, value)?;
            if !(-25.0..=25.0).contains(&value) {
                return Err(invalid(format!("{name} must be between -25 and 25")));
            }
        }
        Ok(())
    }
}

/// Inclusive numeric acceptance interval.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub min: f64,
    pub max: f64,
}

impl Envelope {
    pub fn contains(self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }

    fn validate(self, name: &str) -> SimResult<()> {
        finite(&format!("{name}.min"), self.min)?;
        finite(&format!("{name}.max"), self.max)?;
        if self.min > self.max {
            return Err(invalid(format!("{name}.min must not exceed max")));
        }
        Ok(())
    }
}

/// Aggregate realism targets. These v1 envelopes are provisional model
/// acceptance criteria, not claims about a particular NCAA season.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationEnvelopes {
    pub points_per_team: Envelope,
    pub possessions_per_team: Envelope,
    pub turnovers_per_team: Envelope,
    pub overtime_rate: Envelope,
    pub equal_team_home_win_rate: Envelope,
    pub favorite_win_rate: Envelope,
    pub upset_rate: Envelope,
    pub directional_tolerance: f64,
}

/// All tunable values used by simulation mechanics and calibration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub profile_version: String,
    pub regulation_seconds: u16,
    pub period_seconds: u16,
    pub min_drive_seconds: u16,
    pub max_drive_seconds: u16,
    pub min_plays: u8,
    pub max_plays: u8,
    pub min_yards: i16,
    pub max_yards: i16,
    pub touchdown_weight: f64,
    pub field_goal_weight: f64,
    pub punt_weight: f64,
    pub turnover_weight: f64,
    pub downs_weight: f64,
    pub missed_field_goal_weight: f64,
    pub rating_touchdown_coefficient: f64,
    pub rating_field_goal_coefficient: f64,
    pub rating_turnover_coefficient: f64,
    pub overall_coefficient: f64,
    pub home_advantage_rating: f64,
    pub field_goal_base_rate: f64,
    pub overtime_touchdown_rate: f64,
    pub overtime_field_goal_rate: f64,
    pub max_overtime_rounds: u8,
    pub envelopes: CalibrationEnvelopes,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            profile_version: "provisional-cfb-v1".into(),
            regulation_seconds: 3_600,
            period_seconds: 900,
            min_drive_seconds: 75,
            max_drive_seconds: 210,
            min_plays: 3,
            max_plays: 12,
            min_yards: -10,
            max_yards: 85,
            touchdown_weight: 0.235,
            field_goal_weight: 0.125,
            punt_weight: 0.395,
            turnover_weight: 0.115,
            downs_weight: 0.075,
            missed_field_goal_weight: 0.055,
            rating_touchdown_coefficient: 0.27,
            rating_field_goal_coefficient: 0.10,
            rating_turnover_coefficient: 0.12,
            overall_coefficient: 0.20,
            home_advantage_rating: 4.0,
            field_goal_base_rate: 0.76,
            overtime_touchdown_rate: 0.48,
            overtime_field_goal_rate: 0.30,
            max_overtime_rounds: 20,
            envelopes: CalibrationEnvelopes {
                points_per_team: Envelope {
                    min: 18.0,
                    max: 40.0,
                },
                possessions_per_team: Envelope {
                    min: 8.0,
                    max: 16.0,
                },
                turnovers_per_team: Envelope { min: 0.4, max: 2.5 },
                overtime_rate: Envelope {
                    min: 0.0,
                    max: 0.15,
                },
                equal_team_home_win_rate: Envelope {
                    min: 0.50,
                    max: 0.68,
                },
                favorite_win_rate: Envelope {
                    min: 0.55,
                    max: 0.90,
                },
                upset_rate: Envelope {
                    min: 0.10,
                    max: 0.45,
                },
                directional_tolerance: 0.25,
            },
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> SimResult<()> {
        if self.profile_version.trim().is_empty() {
            return Err(invalid("profile_version cannot be empty"));
        }
        if self.regulation_seconds == 0 || self.period_seconds == 0 {
            return Err(invalid("regulation and period seconds must be positive"));
        }
        if self.regulation_seconds != self.period_seconds.saturating_mul(4) {
            return Err(invalid("regulation_seconds must equal four periods"));
        }
        validate_ordered(
            "drive seconds",
            self.min_drive_seconds,
            self.max_drive_seconds,
        )?;
        validate_ordered("plays", self.min_plays, self.max_plays)?;
        validate_ordered("yards", self.min_yards, self.max_yards)?;
        if self.max_overtime_rounds == 0 {
            return Err(invalid("max_overtime_rounds must be positive"));
        }
        let weights = [
            ("touchdown_weight", self.touchdown_weight),
            ("field_goal_weight", self.field_goal_weight),
            ("punt_weight", self.punt_weight),
            ("turnover_weight", self.turnover_weight),
            ("downs_weight", self.downs_weight),
            ("missed_field_goal_weight", self.missed_field_goal_weight),
        ];
        let mut weight_sum = 0.0;
        for (name, value) in weights {
            probability_component(name, value)?;
            weight_sum += value;
        }
        if weight_sum <= 0.0 {
            return Err(invalid("possession outcome weights must have positive sum"));
        }
        for (name, value) in [
            (
                "rating_touchdown_coefficient",
                self.rating_touchdown_coefficient,
            ),
            (
                "rating_field_goal_coefficient",
                self.rating_field_goal_coefficient,
            ),
            (
                "rating_turnover_coefficient",
                self.rating_turnover_coefficient,
            ),
            ("overall_coefficient", self.overall_coefficient),
            ("home_advantage_rating", self.home_advantage_rating),
        ] {
            finite(name, value)?;
            if value < 0.0 {
                return Err(invalid(format!("{name} cannot be negative")));
            }
        }
        probability("field_goal_base_rate", self.field_goal_base_rate)?;
        probability("overtime_touchdown_rate", self.overtime_touchdown_rate)?;
        probability("overtime_field_goal_rate", self.overtime_field_goal_rate)?;
        if self.overtime_touchdown_rate + self.overtime_field_goal_rate > 1.0 {
            return Err(invalid("overtime scoring probabilities exceed one"));
        }
        for (name, envelope) in [
            ("points_per_team", self.envelopes.points_per_team),
            ("possessions_per_team", self.envelopes.possessions_per_team),
            ("turnovers_per_team", self.envelopes.turnovers_per_team),
            ("overtime_rate", self.envelopes.overtime_rate),
            (
                "equal_team_home_win_rate",
                self.envelopes.equal_team_home_win_rate,
            ),
            ("favorite_win_rate", self.envelopes.favorite_win_rate),
            ("upset_rate", self.envelopes.upset_rate),
        ] {
            envelope.validate(name)?;
        }
        finite(
            "directional_tolerance",
            self.envelopes.directional_tolerance,
        )?;
        if self.envelopes.directional_tolerance < 0.0 {
            return Err(invalid("directional_tolerance cannot be negative"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PossessionOutcome {
    Touchdown,
    FieldGoal,
    MissedFieldGoal,
    Punt,
    Turnover,
    TurnoverOnDowns,
    EndOfPeriod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Possession {
    pub index: u16,
    pub offense_team_id: String,
    pub period: Quarter,
    pub start_seconds_remaining: u16,
    pub duration_seconds: u16,
    pub start_yard_line: u8,
    pub end_yard_line: u8,
    pub outcome: PossessionOutcome,
    pub points: u8,
    pub plays: u8,
    pub yards: i16,
    pub passing_yards: i16,
    pub rushing_yards: i16,
    pub first_downs: u8,
    pub third_down_attempts: u8,
    pub third_down_conversions: u8,
    pub field_goal_attempts: u8,
    pub field_goals_made: u8,
    pub punts: u8,
    pub turnovers: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamGameStats {
    pub points: u16,
    pub possessions: u16,
    pub plays: u16,
    pub total_yards: i32,
    pub passing_yards: i32,
    pub rushing_yards: i32,
    pub turnovers: u16,
    pub first_downs: u16,
    pub third_down_attempts: u16,
    pub third_down_conversions: u16,
    pub field_goal_attempts: u16,
    pub field_goals_made: u16,
    pub punts: u16,
    pub possession_seconds: u32,
}

impl TeamGameStats {
    pub fn third_down_rate(&self) -> Option<f64> {
        (self.third_down_attempts > 0)
            .then(|| self.third_down_conversions as f64 / self.third_down_attempts as f64)
    }

    fn add(&mut self, possession: &Possession) {
        self.points += u16::from(possession.points);
        self.possessions += 1;
        self.plays += u16::from(possession.plays);
        self.total_yards += i32::from(possession.yards);
        self.passing_yards += i32::from(possession.passing_yards);
        self.rushing_yards += i32::from(possession.rushing_yards);
        self.turnovers += u16::from(possession.turnovers);
        self.first_downs += u16::from(possession.first_downs);
        self.third_down_attempts += u16::from(possession.third_down_attempts);
        self.third_down_conversions += u16::from(possession.third_down_conversions);
        self.field_goal_attempts += u16::from(possession.field_goal_attempts);
        self.field_goals_made += u16::from(possession.field_goals_made);
        self.punts += u16::from(possession.punts);
        self.possession_seconds += u32::from(possession.duration_seconds);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationProvenance {
    pub algorithm_version: String,
    pub profile_version: String,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub game: Game,
    pub possessions: Vec<Possession>,
    pub home_stats: TeamGameStats,
    pub away_stats: TeamGameStats,
    pub overtime_rounds: u8,
    pub provenance: SimulationProvenance,
}

impl SimulationResult {
    pub fn validate(&self) -> SimResult<()> {
        if !self.game.is_completed() || self.game.is_tie() {
            return Err(SimError::SimulationError(
                "simulated game must be completed and untied".into(),
            ));
        }
        let mut home = TeamGameStats::default();
        let mut away = TeamGameStats::default();
        for (expected_index, possession) in self.possessions.iter().enumerate() {
            if usize::from(possession.index) != expected_index {
                return Err(SimError::SimulationError(
                    "possession indexes are not contiguous".into(),
                ));
            }
            if possession.start_yard_line > 100 || possession.end_yard_line > 100 {
                return Err(SimError::SimulationError(
                    "possession field position is out of bounds".into(),
                ));
            }
            if possession.offense_team_id == self.game.home_team.id {
                home.add(possession);
            } else if possession.offense_team_id == self.game.away_team.id {
                away.add(possession);
            } else {
                return Err(SimError::SimulationError(
                    "possession references an unknown team".into(),
                ));
            }
        }
        if home != self.home_stats || away != self.away_stats {
            return Err(SimError::SimulationError(
                "team summaries do not reconcile with possessions".into(),
            ));
        }
        if home.points != self.game.home_score.total || away.points != self.game.away_score.total {
            return Err(SimError::SimulationError(
                "summary points do not reconcile with final score".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Home,
    Away,
}

impl Side {
    fn other(self) -> Self {
        match self {
            Self::Home => Self::Away,
            Self::Away => Self::Home,
        }
    }
}

struct GameSimulationState {
    elapsed: u16,
    offense: Side,
    start_yard_line: u8,
    possessions: Vec<Possession>,
    home_stats: TeamGameStats,
    away_stats: TeamGameStats,
}

impl GameSimulationState {
    fn new(first_offense: Side) -> Self {
        Self {
            elapsed: 0,
            offense: first_offense,
            start_yard_line: 25,
            possessions: Vec::new(),
            home_stats: TeamGameStats::default(),
            away_stats: TeamGameStats::default(),
        }
    }

    fn add(&mut self, possession: Possession) {
        match self.offense {
            Side::Home => self.home_stats.add(&possession),
            Side::Away => self.away_stats.add(&possession),
        }
        self.possessions.push(possession);
    }
}

/// Simulates a complete game with no I/O or global state.
pub fn simulate_game(
    matchup: &Matchup,
    config: &SimulationConfig,
    seed: u64,
) -> SimResult<SimulationResult> {
    matchup.validate()?;
    config.validate()?;

    let mut coin_rng = SimRng::new(derive_seed(seed, "coin-toss"));
    let mut regulation_rng = SimRng::new(derive_seed(seed, "regulation"));
    let mut overtime_rng = SimRng::new(derive_seed(seed, "overtime"));
    let first = if coin_rng.bool() {
        Side::Home
    } else {
        Side::Away
    };
    let mut state = GameSimulationState::new(first);

    while state.elapsed < config.regulation_seconds {
        let previous_elapsed = state.elapsed;
        let possession = sample_possession(matchup, config, &state, &mut regulation_rng);
        let outcome = possession.outcome;
        let end_yard_line = possession.end_yard_line;
        state.elapsed = state
            .elapsed
            .saturating_add(possession.duration_seconds)
            .min(config.regulation_seconds);
        state.add(possession);
        if previous_elapsed < config.period_seconds * 2
            && state.elapsed >= config.period_seconds * 2
        {
            state.start_yard_line = 25;
            state.offense = first.other();
        } else {
            state.start_yard_line = next_start(outcome, end_yard_line, &mut regulation_rng);
            state.offense = state.offense.other();
        }
    }

    let regulation_home = state.home_stats.points;
    let regulation_away = state.away_stats.points;
    let overtime_rounds = if regulation_home == regulation_away {
        play_overtime(matchup, config, &mut state, &mut overtime_rng)
    } else {
        0
    };

    let mut game = Game::new(
        matchup.game_id.clone(),
        matchup.home.clone(),
        matchup.away.clone(),
        matchup.location.clone(),
        matchup.week,
        matchup.conference_game,
        matchup.venue == Venue::Neutral,
    );
    game.start()?;
    for possession in &state.possessions {
        if possession.points == 0 {
            continue;
        }
        let score = if possession.offense_team_id == matchup.home.id {
            &mut game.home_score
        } else {
            &mut game.away_score
        };
        score.add_points(possession.period, u16::from(possession.points));
    }
    game.complete()?;

    let result = SimulationResult {
        game,
        possessions: state.possessions,
        home_stats: state.home_stats,
        away_stats: state.away_stats,
        overtime_rounds,
        provenance: SimulationProvenance {
            algorithm_version: ALGORITHM_VERSION.into(),
            profile_version: config.profile_version.clone(),
            seed,
        },
    };
    result.validate()?;
    Ok(result)
}

fn sample_possession(
    matchup: &Matchup,
    config: &SimulationConfig,
    state: &GameSimulationState,
    rng: &mut SimRng,
) -> Possession {
    let remaining = config.regulation_seconds - state.elapsed;
    let duration = rng
        .int_range(
            config.min_drive_seconds,
            config.max_drive_seconds.saturating_add(1),
        )
        .min(remaining);
    let period_index = (state.elapsed / config.period_seconds).min(3);
    let period = quarter(period_index);
    let seconds_into_period = state.elapsed % config.period_seconds;
    let start_seconds_remaining = config.period_seconds - seconds_into_period;
    let (offense, defense) = teams(matchup, state.offense);
    let efficiency = matchup_efficiency(matchup, config, state.offense);
    let weights = outcome_weights(config, efficiency);
    let selected = rng.weighted_index(&weights).unwrap_or(2);
    let outcome = match selected {
        0 => PossessionOutcome::Touchdown,
        1 => PossessionOutcome::FieldGoal,
        2 => PossessionOutcome::Punt,
        3 => PossessionOutcome::Turnover,
        4 => PossessionOutcome::TurnoverOnDowns,
        _ => PossessionOutcome::MissedFieldGoal,
    };
    let plays = rng.int_range(config.min_plays, config.max_plays.saturating_add(1));
    let expected_yards = (efficiency * 18.0).round() as i16;
    let mut yards = rng
        .int_range(config.min_yards, config.max_yards.saturating_add(1))
        .saturating_add(expected_yards)
        .clamp(config.min_yards, config.max_yards);
    if outcome == PossessionOutcome::Touchdown {
        yards = i16::from(100 - state.start_yard_line);
    }
    let end_yard_line = (i16::from(state.start_yard_line) + yards).clamp(0, 100) as u8;
    let passing_share = rng.int_range(45_u16, 71_u16);
    let passing_yards = ((i32::from(yards) * i32::from(passing_share)) / 100) as i16;
    let rushing_yards = yards - passing_yards;
    let third_down_attempts = plays / 3;
    let conversion_base = (0.38 + efficiency * 0.12).clamp(0.15, 0.70);
    let third_down_conversions = (0..third_down_attempts)
        .filter(|_| rng.float() < conversion_base)
        .count() as u8;
    let first_downs = ((yards.max(0) as u16 / 12) as u8)
        .max(third_down_conversions)
        .min(plays);
    let field_goal_attempts = u8::from(matches!(
        outcome,
        PossessionOutcome::FieldGoal | PossessionOutcome::MissedFieldGoal
    ));
    let field_goals_made = u8::from(outcome == PossessionOutcome::FieldGoal);
    let points = match outcome {
        PossessionOutcome::Touchdown => 7,
        PossessionOutcome::FieldGoal => 3,
        _ => 0,
    };
    let _ = (offense, defense); // Names clarify the rating matchup above.
    Possession {
        index: state.possessions.len() as u16,
        offense_team_id: offense.id.clone(),
        period,
        start_seconds_remaining,
        duration_seconds: duration,
        start_yard_line: state.start_yard_line,
        end_yard_line,
        outcome,
        points,
        plays,
        yards,
        passing_yards,
        rushing_yards,
        first_downs,
        third_down_attempts,
        third_down_conversions,
        field_goal_attempts,
        field_goals_made,
        punts: u8::from(outcome == PossessionOutcome::Punt),
        turnovers: u8::from(outcome == PossessionOutcome::Turnover),
    }
}

fn play_overtime(
    matchup: &Matchup,
    config: &SimulationConfig,
    state: &mut GameSimulationState,
    rng: &mut SimRng,
) -> u8 {
    for round in 1..=config.max_overtime_rounds {
        let first = if rng.bool() { Side::Home } else { Side::Away };
        for side in [first, first.other()] {
            let points = overtime_points(config, rng);
            let team_id = match side {
                Side::Home => matchup.home.id.clone(),
                Side::Away => matchup.away.id.clone(),
            };
            let outcome = match points {
                7 => PossessionOutcome::Touchdown,
                3 => PossessionOutcome::FieldGoal,
                _ => PossessionOutcome::TurnoverOnDowns,
            };
            let possession = Possession {
                index: state.possessions.len() as u16,
                offense_team_id: team_id,
                period: Quarter::Overtime(round),
                start_seconds_remaining: 0,
                duration_seconds: 0,
                start_yard_line: 75,
                end_yard_line: if points == 7 { 100 } else { 85 },
                outcome,
                points,
                plays: 5,
                yards: if points == 7 { 25 } else { 10 },
                passing_yards: if points == 7 { 15 } else { 6 },
                rushing_yards: if points == 7 { 10 } else { 4 },
                first_downs: u8::from(points > 0),
                third_down_attempts: 1,
                third_down_conversions: u8::from(points == 7),
                field_goal_attempts: u8::from(points == 3),
                field_goals_made: u8::from(points == 3),
                punts: 0,
                turnovers: 0,
            };
            match side {
                Side::Home => state.home_stats.add(&possession),
                Side::Away => state.away_stats.add(&possession),
            }
            state.possessions.push(possession);
        }
        if state.home_stats.points != state.away_stats.points {
            return round;
        }
    }
    // The finite cap protects callers from pathological configurations while
    // preserving an untied result deterministically.
    let winner = if rng.bool() { Side::Home } else { Side::Away };
    let team_id = match winner {
        Side::Home => matchup.home.id.clone(),
        Side::Away => matchup.away.id.clone(),
    };
    let possession = Possession {
        index: state.possessions.len() as u16,
        offense_team_id: team_id,
        period: Quarter::Overtime(config.max_overtime_rounds.saturating_add(1)),
        start_seconds_remaining: 0,
        duration_seconds: 0,
        start_yard_line: 97,
        end_yard_line: 100,
        outcome: PossessionOutcome::Touchdown,
        points: 2,
        plays: 1,
        yards: 3,
        passing_yards: 2,
        rushing_yards: 1,
        first_downs: 0,
        third_down_attempts: 0,
        third_down_conversions: 0,
        field_goal_attempts: 0,
        field_goals_made: 0,
        punts: 0,
        turnovers: 0,
    };
    match winner {
        Side::Home => state.home_stats.add(&possession),
        Side::Away => state.away_stats.add(&possession),
    }
    state.possessions.push(possession);
    config.max_overtime_rounds.saturating_add(1)
}

fn overtime_points(config: &SimulationConfig, rng: &mut SimRng) -> u8 {
    let draw = rng.float();
    if draw < config.overtime_touchdown_rate {
        7
    } else if draw < config.overtime_touchdown_rate + config.overtime_field_goal_rate {
        3
    } else {
        0
    }
}

fn outcome_weights(config: &SimulationConfig, efficiency: f64) -> [f64; 6] {
    [
        (config.touchdown_weight + efficiency * config.rating_touchdown_coefficient).max(0.001),
        (config.field_goal_weight + efficiency * config.rating_field_goal_coefficient).max(0.001),
        config.punt_weight.max(0.001),
        (config.turnover_weight - efficiency * config.rating_turnover_coefficient).max(0.001),
        config.downs_weight.max(0.001),
        config.missed_field_goal_weight.max(0.001),
    ]
}

fn matchup_efficiency(matchup: &Matchup, config: &SimulationConfig, side: Side) -> f64 {
    let (offense, defense) = teams(matchup, side);
    let (offense_modifier, defense_modifier) = match side {
        Side::Home => (
            matchup.modifiers.home_offense,
            matchup.modifiers.away_defense,
        ),
        Side::Away => (
            matchup.modifiers.away_offense,
            matchup.modifiers.home_defense,
        ),
    };
    let unit = (f64::from(offense.offense_rating) + offense_modifier
        - f64::from(defense.defense_rating)
        - defense_modifier)
        / 100.0;
    let overall = (f64::from(offense.rating) - f64::from(defense.rating)) / 100.0
        * config.overall_coefficient;
    let (offense_special, defense_special) = match side {
        Side::Home => (
            matchup.modifiers.home_special_teams,
            matchup.modifiers.away_special_teams,
        ),
        Side::Away => (
            matchup.modifiers.away_special_teams,
            matchup.modifiers.home_special_teams,
        ),
    };
    let special = (f64::from(offense.special_teams_rating) + offense_special
        - f64::from(defense.special_teams_rating)
        - defense_special)
        / 500.0;
    let venue = if side == Side::Home && matchup.venue == Venue::Home {
        config.home_advantage_rating / 100.0
    } else {
        0.0
    };
    (unit + overall + special + venue).clamp(-0.55, 0.55)
}

fn teams(matchup: &Matchup, side: Side) -> (&Team, &Team) {
    match side {
        Side::Home => (&matchup.home, &matchup.away),
        Side::Away => (&matchup.away, &matchup.home),
    }
}

fn next_start(outcome: PossessionOutcome, end: u8, rng: &mut SimRng) -> u8 {
    match outcome {
        PossessionOutcome::Touchdown | PossessionOutcome::FieldGoal => 25,
        PossessionOutcome::Punt => {
            let punt = rng.int_range(35_u8, 51_u8);
            (100_i16 - i16::from(end) - i16::from(punt)).clamp(5, 80) as u8
        }
        PossessionOutcome::Turnover
        | PossessionOutcome::TurnoverOnDowns
        | PossessionOutcome::MissedFieldGoal => (100_u8.saturating_sub(end)).clamp(5, 95),
        PossessionOutcome::EndOfPeriod => 25,
    }
}

fn quarter(index: u16) -> Quarter {
    match index {
        0 => Quarter::First,
        1 => Quarter::Second,
        2 => Quarter::Third,
        _ => Quarter::Fourth,
    }
}

/// Derives a stable substream seed with FNV-1a followed by SplitMix64 finalization.
pub fn derive_seed(seed: u64, label: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64 ^ seed;
    for byte in label.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let mut value = hash.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn probability(name: &str, value: f64) -> SimResult<()> {
    finite(name, value)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(invalid(format!("{name} must be between zero and one")));
    }
    Ok(())
}

fn probability_component(name: &str, value: f64) -> SimResult<()> {
    finite(name, value)?;
    if value < 0.0 {
        return Err(invalid(format!("{name} cannot be negative")));
    }
    Ok(())
}

fn finite(name: &str, value: f64) -> SimResult<()> {
    if !value.is_finite() {
        return Err(invalid(format!("{name} must be finite")));
    }
    Ok(())
}

fn validate_ordered<T: PartialOrd>(name: &str, min: T, max: T) -> SimResult<()> {
    if min > max {
        return Err(invalid(format!("{name} minimum must not exceed maximum")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> SimError {
    SimError::InvalidParameter(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::depth_chart::UnitStrengths;
    use crate::player::{ClassYear, Eligibility, Player, PlayerAttributes, Position};
    use crate::roster::Roster;

    fn team(id: &str, rating: u8) -> Team {
        Team::new(
            id,
            format!("Team {id}"),
            id.to_uppercase(),
            "Mascot",
            "Test",
            None,
            "Testville",
            rating,
            rating,
            rating,
            rating,
        )
        .expect("valid fixture")
    }

    fn matchup() -> Matchup {
        Matchup {
            game_id: "game-1".into(),
            home: team("home", 80),
            away: team("away", 75),
            location: "Stadium".into(),
            week: 1,
            conference_game: true,
            venue: Venue::Home,
            modifiers: MatchupModifiers::default(),
        }
    }

    #[test]
    fn default_profile_is_valid_and_serializable() {
        let config = SimulationConfig::default();
        config.validate().expect("default profile is valid");
        let json = serde_json::to_string(&config).expect("serialize");
        let decoded: SimulationConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config, decoded);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let config = SimulationConfig {
            touchdown_weight: f64::NAN,
            ..SimulationConfig::default()
        };
        assert!(config.validate().is_err());
        let config = SimulationConfig {
            min_plays: 12,
            max_plays: 3,
            ..SimulationConfig::default()
        };
        assert!(config.validate().is_err());
        let mut invalid_matchup = matchup();
        invalid_matchup.away.id = invalid_matchup.home.id.clone();
        assert!(simulate_game(&invalid_matchup, &SimulationConfig::default(), 1).is_err());
    }

    #[test]
    fn labeled_seed_derivation_is_stable_and_isolated() {
        assert_eq!(derive_seed(42, "regulation"), derive_seed(42, "regulation"));
        assert_ne!(derive_seed(42, "regulation"), derive_seed(42, "overtime"));
        assert_ne!(derive_seed(42, "regulation"), derive_seed(43, "regulation"));
    }

    #[test]
    fn simulation_replays_and_reconciles() {
        let config = SimulationConfig::default();
        let first = simulate_game(&matchup(), &config, 42).expect("simulation succeeds");
        let second = simulate_game(&matchup(), &config, 42).expect("simulation succeeds");
        assert_eq!(first.possessions, second.possessions);
        assert_eq!(first.home_stats, second.home_stats);
        assert_eq!(first.away_stats, second.away_stats);
        assert_eq!(first.game.home_score, second.game.home_score);
        assert_eq!(first.game.away_score, second.game.away_score);
        first.validate().expect("result reconciles");
    }

    #[test]
    fn adjacent_roster_does_not_change_seeded_aggregate_simulation() {
        let config = SimulationConfig::default();
        let before = simulate_game(&matchup(), &config, 42).unwrap();
        let _adjacent_roster = Roster::new(
            "home",
            2026,
            vec![Player::new(
                "player-1",
                "Ada",
                "Lovelace",
                Position::Quarterback,
                PlayerAttributes::new(90, 60, 85, 92, 88).unwrap(),
                Eligibility::new(ClassYear::Freshman, 0, 4, false).unwrap(),
            )
            .unwrap()],
        )
        .unwrap();
        let after = simulate_game(&matchup(), &config, 42).unwrap();
        assert_eq!(before.game.home_score, after.game.home_score);
        assert_eq!(before.game.away_score, after.game.away_score);
        assert_eq!(before.possessions, after.possessions);
    }

    #[test]
    fn neutral_roster_strengths_preserve_default_seeded_result() {
        let config = SimulationConfig::default();
        let baseline = simulate_game(&matchup(), &config, 42).unwrap();
        let neutral = UnitStrengths {
            offense: 50,
            defense: 50,
            special_teams: 50,
        };
        let mut composed = matchup();
        composed.modifiers = UnitStrengths::matchup_modifiers(neutral, neutral);
        let result = simulate_game(&composed, &config, 42).unwrap();
        assert_eq!(baseline.possessions, result.possessions);
        assert_eq!(baseline.game.home_score, result.game.home_score);
        assert_eq!(baseline.game.away_score, result.game.away_score);
    }

    #[test]
    fn roster_derived_unit_modifiers_have_directional_paired_seed_effects() {
        let config = SimulationConfig::default();
        let neutral = UnitStrengths {
            offense: 50,
            defense: 50,
            special_teams: 50,
        };
        let mut baseline = matchup();
        baseline.home = team("home", 75);
        baseline.away = team("away", 75);
        baseline.venue = Venue::Neutral;
        baseline.modifiers = UnitStrengths::matchup_modifiers(neutral, neutral);

        let mean = |strengths: UnitStrengths| {
            let mut candidate = baseline.clone();
            candidate.modifiers = UnitStrengths::matchup_modifiers(strengths, neutral);
            (0..400).fold((0_u64, 0_u64), |totals, seed| {
                let result = simulate_game(&candidate, &config, seed).unwrap();
                (
                    totals.0 + u64::from(result.home_stats.points),
                    totals.1 + u64::from(result.away_stats.points),
                )
            })
        };
        let base = mean(neutral);
        let offense = mean(UnitStrengths {
            offense: 100,
            ..neutral
        });
        let defense = mean(UnitStrengths {
            defense: 100,
            ..neutral
        });
        let special = mean(UnitStrengths {
            special_teams: 100,
            ..neutral
        });
        assert!(offense.0 >= base.0);
        assert!(defense.1 <= base.1);
        assert!(special.0.saturating_sub(special.1) >= base.0.saturating_sub(base.1));
    }

    #[test]
    fn different_seeds_vary_results() {
        let config = SimulationConfig::default();
        let scores = (0..12)
            .map(|seed| {
                let result = simulate_game(&matchup(), &config, seed).expect("simulation succeeds");
                (result.home_stats.points, result.away_stats.points)
            })
            .collect::<std::collections::HashSet<_>>();
        assert!(scores.len() > 1);
    }

    #[test]
    fn zero_attempt_rate_is_absent() {
        assert_eq!(TeamGameStats::default().third_down_rate(), None);
    }

    #[test]
    fn higher_ratings_improve_paired_seed_results() {
        let config = SimulationConfig::default();
        let mut weak = matchup();
        weak.home = team("home", 55);
        weak.away = team("away", 75);
        weak.venue = Venue::Neutral;
        let mut strong = weak.clone();
        strong.home = team("home", 95);
        let weak_points: u32 = (0..250)
            .map(|seed| {
                u32::from(
                    simulate_game(&weak, &config, seed)
                        .unwrap()
                        .home_stats
                        .points,
                )
            })
            .sum();
        let strong_points: u32 = (0..250)
            .map(|seed| {
                u32::from(
                    simulate_game(&strong, &config, seed)
                        .unwrap()
                        .home_stats
                        .points,
                )
            })
            .sum();
        assert!(strong_points >= weak_points);
    }

    #[test]
    fn venue_and_unit_ratings_have_directional_effects() {
        let config = SimulationConfig::default();
        let base = matchup();
        let mut neutral = base.clone();
        neutral.venue = Venue::Neutral;
        let mut offense = neutral.clone();
        offense.home.offense_rating = 95;
        let mut defense = neutral.clone();
        defense.home.defense_rating = 95;
        let mut special = neutral.clone();
        special.home.special_teams_rating = 95;

        let aggregate = |candidate: &Matchup| {
            (0..300).fold((0_u64, 0_u64, 0_u64), |acc, seed| {
                let result = simulate_game(candidate, &config, seed).unwrap();
                (
                    acc.0 + u64::from(result.home_stats.points),
                    acc.1 + u64::from(result.away_stats.points),
                    acc.2 + u64::from(result.home_stats.field_goals_made),
                )
            })
        };
        let baseline = aggregate(&neutral);
        let home_venue = aggregate(&base);
        let offense_stronger = aggregate(&offense);
        let defense_stronger = aggregate(&defense);
        let special_stronger = aggregate(&special);
        assert!(home_venue.0 >= baseline.0);
        assert!(offense_stronger.0 >= baseline.0);
        assert!(defense_stronger.1 <= baseline.1);
        assert!(special_stronger.0 >= baseline.0 || special_stronger.2 >= baseline.2);
    }

    #[test]
    fn all_outcomes_and_bounds_are_exercised() {
        let config = SimulationConfig::default();
        let mut outcomes = std::collections::HashSet::new();
        for seed in 0..100 {
            let result = simulate_game(&matchup(), &config, seed).unwrap();
            for possession in result.possessions {
                assert!(possession.start_yard_line <= 100);
                assert!(possession.end_yard_line <= 100);
                outcomes.insert(possession.outcome as u8);
            }
        }
        assert!(outcomes.len() >= 6);
    }
}
