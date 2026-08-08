//! cli: Command-line interface entrypoint for the simulator workspace.
//!
//! Commands in this binary provide a lightweight harness for exercising features
//! as they are added to the domain and persistence crates.

use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use sim_core::calibration::run_calibration;
use sim_core::game::{Game, Quarter};
use sim_core::rng::SimRng;
use sim_core::season::{ScheduledGame, Season, TeamRecord};
use sim_core::simulation::{
    simulate_game, Matchup, MatchupModifiers, SimulationConfig, SimulationResult, Venue,
};
use sim_core::team::Team;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

/// College Football Simulator CLI
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Feature harness for the college football simulator"
)]
struct Cli {
    /// Increase verbosity (-v debug, -vv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Verify that each workspace component is linked and available
    Health,
    /// Print a reproducible sequence from the simulator RNG
    Rng {
        /// Seed used to initialize the deterministic RNG
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Number of values to generate
        #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u16).range(1..=1000))]
        count: u16,
        /// Exclusive upper bound for each generated value
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..))]
        max: u32,
    },
    /// Exercise game creation, scoring, lifecycle, and winner selection
    Game {
        /// Final score for the home team
        #[arg(long)]
        home_score: u16,
        /// Final score for the away team
        #[arg(long)]
        away_score: u16,
        /// Mark the game as a conference matchup
        #[arg(long)]
        conference: bool,
    },
    /// Exercise completed-game record calculation and standings
    Season {
        /// Final score for the home team
        #[arg(long)]
        home_score: u16,
        /// Final score for the away team
        #[arg(long)]
        away_score: u16,
    },
    /// Display the validated sample regular-season schedule and derived game seeds
    Schedule {
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    /// Simulate the current week or the full sample regular season
    SeasonLoop {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Advance every remaining regular-season week
        #[arg(long)]
        full: bool,
    },
    /// Simulate a reproducible rated matchup possession by possession
    Simulate {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long)]
        neutral: bool,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        home_rating: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        home_offense: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        home_defense: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        home_special_teams: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        away_rating: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        away_offense: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        away_defense: u8,
        #[arg(long, default_value_t = 75, value_parser = clap::value_parser!(u8).range(0..=100))]
        away_special_teams: u8,
    },
    /// Run deterministic aggregate statistical calibration
    Calibrate {
        #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u32).range(1..))]
        seeds: u32,
        /// Emit the machine-readable report as JSON
        #[arg(long)]
        json: bool,
    },
}

fn init_tracing(verbose: u8) {
    // Map -v levels to a filter level string
    let level: Level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    // Allow RUST_LOG to override, default to chosen level
    let default_directive = format!("{}{}", level, "");
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_directive));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn main() -> Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbose);

    info!(command = ?args.command, "running CLI command");
    println!("{}", execute(args.command)?);

    Ok(())
}

fn execute(command: Command) -> Result<String> {
    match command {
        Command::Health => Ok(format!(
            "sim_core={} v{}\npersistence={}",
            sim_core::ping(),
            sim_core::VERSION,
            persistence::ping()
        )),
        Command::Rng { seed, count, max } => {
            ensure!(count > 0, "count must be greater than zero");
            ensure!(max > 0, "max must be greater than zero");
            let mut rng = SimRng::new(seed);
            let values = (0..count)
                .map(|_| rng.int(max).to_string())
                .collect::<Vec<_>>()
                .join(",");
            Ok(format!("seed={seed}\nvalues={values}"))
        }
        Command::Game {
            home_score,
            away_score,
            conference,
        } => {
            let game = completed_game(home_score, away_score, conference)?;
            let outcome = game
                .winner()
                .map(|team| format!("winner={}", team.name))
                .unwrap_or_else(|| "winner=tie".to_string());
            Ok(format!("{game}\n{outcome}"))
        }
        Command::Season {
            home_score,
            away_score,
        } => {
            let (home, away) = records_for_score(home_score, away_score);
            Ok(format!("Home State: {home}\nAway Tech: {away}"))
        }
        Command::Schedule { seed } => {
            let season = sample_season()?;
            Ok(format_schedule(&season, seed)?)
        }
        Command::SeasonLoop { seed, full } => run_season_loop(seed, full),
        Command::Simulate {
            seed,
            neutral,
            home_rating,
            home_offense,
            home_defense,
            home_special_teams,
            away_rating,
            away_offense,
            away_defense,
            away_special_teams,
        } => {
            let home = rated_team(
                "home",
                "Home State",
                "HST",
                home_rating,
                home_offense,
                home_defense,
                home_special_teams,
            )?;
            let away = rated_team(
                "away",
                "Away Tech",
                "AWT",
                away_rating,
                away_offense,
                away_defense,
                away_special_teams,
            )?;
            let matchup = Matchup {
                game_id: "cli-simulation".into(),
                home,
                away,
                location: if neutral {
                    "Neutral Site"
                } else {
                    "Home Stadium"
                }
                .into(),
                week: 1,
                conference_game: false,
                venue: if neutral { Venue::Neutral } else { Venue::Home },
                modifiers: MatchupModifiers::default(),
            };
            let result = simulate_game(&matchup, &SimulationConfig::default(), seed)?;
            Ok(format_simulation(&result))
        }
        Command::Calibrate { seeds, json } => {
            let report = run_calibration(&SimulationConfig::default(), seeds)?;
            let output = if json {
                serde_json::to_string_pretty(&report)?
            } else {
                format_calibration(&report)
            };
            ensure!(report.passed, "calibration failed\n{output}");
            Ok(output)
        }
    }
}

fn records_for_score(home_score: u16, away_score: u16) -> (TeamRecord, TeamRecord) {
    let mut home = TeamRecord::new();
    let mut away = TeamRecord::new();
    match home_score.cmp(&away_score) {
        std::cmp::Ordering::Greater => {
            home.wins = 1;
            home.conference_wins = 1;
            away.losses = 1;
            away.conference_losses = 1;
        }
        std::cmp::Ordering::Less => {
            away.wins = 1;
            away.conference_wins = 1;
            home.losses = 1;
            home.conference_losses = 1;
        }
        std::cmp::Ordering::Equal => {
            home.ties = 1;
            home.conference_ties = 1;
            away.ties = 1;
            away.conference_ties = 1;
        }
    }
    (home, away)
}

fn sample_season() -> Result<Season> {
    let teams = vec![
        rated_team("home", "Home State", "HST", 82, 84, 81, 76)?,
        rated_team("away", "Away Tech", "AWT", 76, 78, 74, 72)?,
        rated_team("north", "North College", "NTH", 79, 80, 79, 77)?,
        rated_team("south", "South University", "STH", 73, 74, 72, 75)?,
    ];
    let games = vec![
        ScheduledGame::new(
            "2026-w1-away-at-home",
            "home",
            "away",
            "Home Stadium",
            1,
            true,
            Venue::Home,
        ),
        ScheduledGame::new(
            "2026-w1-south-at-north",
            "north",
            "south",
            "North Stadium",
            1,
            true,
            Venue::Home,
        ),
        ScheduledGame::new(
            "2026-w2-north-at-home",
            "home",
            "north",
            "Home Stadium",
            2,
            false,
            Venue::Home,
        ),
        ScheduledGame::new(
            "2026-w2-away-vs-south",
            "away",
            "south",
            "Kickoff Classic",
            2,
            false,
            Venue::Neutral,
        ),
        ScheduledGame::new(
            "2026-w3-home-at-south",
            "south",
            "home",
            "South Stadium",
            3,
            true,
            Venue::Home,
        ),
        ScheduledGame::new(
            "2026-w3-north-at-away",
            "away",
            "north",
            "Away Stadium",
            3,
            true,
            Venue::Home,
        ),
    ];
    Ok(Season::new(2026, teams, 3, games)?)
}

fn format_schedule(season: &Season, seed: u64) -> Result<String> {
    season
        .schedule()
        .entries()
        .iter()
        .map(|game| {
            Ok(format!(
                "week={} game={} away={} home={} venue={:?} conference={} seed={}",
                game.week,
                game.id,
                game.away_team_id,
                game.home_team_id,
                game.venue,
                game.is_conference_game,
                season.game_seed(seed, &game.id)?
            ))
        })
        .collect::<Result<Vec<_>>>()
        .map(|lines| lines.join("\n"))
}

fn run_season_loop(seed: u64, full: bool) -> Result<String> {
    let mut season = sample_season()?;
    let config = SimulationConfig::default();
    if full {
        season.advance_regular_season(seed, &config)?;
    } else {
        season.advance_week(seed, &config)?;
    }

    let results = season
        .schedule()
        .entries()
        .iter()
        .filter_map(|scheduled| season.result_for_game(&scheduled.id))
        .map(|result| {
            format!(
                "week={} game={} {} {} - {} {} model={} profile={} seed={}",
                result.game.week,
                result.game.id,
                result.game.away_team.name,
                result.game.away_score.total,
                result.game.home_score.total,
                result.game.home_team.name,
                result.provenance.algorithm_version,
                result.provenance.profile_version,
                result.provenance.seed,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let standings = season
        .conference_standings("Demo Conference")
        .iter()
        .map(|(team, record)| {
            format!(
                "{} overall={} conference={}",
                team.name,
                record,
                record.conference_to_string()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "season=2026 seed={seed} state={} next_week={}\nResults\n{results}\nStandings\n{standings}",
        if season.is_complete() {
            "complete"
        } else {
            "in-progress"
        },
        season
            .current_week()
            .map_or_else(|| "none".into(), |week| week.to_string())
    ))
}

fn rated_team(
    id: &str,
    name: &str,
    abbreviation: &str,
    rating: u8,
    offense: u8,
    defense: u8,
    special_teams: u8,
) -> Result<Team> {
    Ok(Team::new(
        id,
        name,
        abbreviation,
        name,
        "Demo Conference",
        None,
        name,
        rating,
        offense,
        defense,
        special_teams,
    )?)
}

fn format_simulation(result: &SimulationResult) -> String {
    let home = &result.home_stats;
    let away = &result.away_stats;
    format!(
        "{} {} - {} {} (Final)\nPeriods: away {}-{}-{}-{}+{} | home {}-{}-{}-{}+{}\n\
         Stat                  Away   Home\n\
         Possessions           {:>3}    {:>3}\n\
         Plays                 {:>3}    {:>3}\n\
         Total yards           {:>3}    {:>3}\n\
         Passing yards         {:>3}    {:>3}\n\
         Rushing yards         {:>3}    {:>3}\n\
         First downs           {:>3}    {:>3}\n\
         Turnovers             {:>3}    {:>3}\n\
         Third down          {:>2}/{:<2}  {:>2}/{:<2}\n\
         Field goals         {:>2}/{:<2}  {:>2}/{:<2}\n\
         Punts                 {:>3}    {:>3}\n\
         Possession seconds   {:>4}   {:>4}\n\
         model={} profile={} seed={} overtime_rounds={}",
        result.game.away_team.name,
        away.points,
        home.points,
        result.game.home_team.name,
        result.game.away_score.q1,
        result.game.away_score.q2,
        result.game.away_score.q3,
        result.game.away_score.q4,
        result.game.away_score.ot,
        result.game.home_score.q1,
        result.game.home_score.q2,
        result.game.home_score.q3,
        result.game.home_score.q4,
        result.game.home_score.ot,
        away.possessions,
        home.possessions,
        away.plays,
        home.plays,
        away.total_yards,
        home.total_yards,
        away.passing_yards,
        home.passing_yards,
        away.rushing_yards,
        home.rushing_yards,
        away.first_downs,
        home.first_downs,
        away.turnovers,
        home.turnovers,
        away.third_down_conversions,
        away.third_down_attempts,
        home.third_down_conversions,
        home.third_down_attempts,
        away.field_goals_made,
        away.field_goal_attempts,
        home.field_goals_made,
        home.field_goal_attempts,
        away.punts,
        home.punts,
        away.possession_seconds,
        home.possession_seconds,
        result.provenance.algorithm_version,
        result.provenance.profile_version,
        result.provenance.seed,
        result.overtime_rounds,
    )
}

fn format_calibration(report: &sim_core::calibration::CalibrationReport) -> String {
    let metrics = report
        .metrics
        .iter()
        .map(|metric| {
            format!(
                "{}={:.4} expected=[{:.4},{:.4}] n={} {}",
                metric.name,
                metric.observed,
                metric.expected.min,
                metric.expected.max,
                metric.sample_size,
                if metric.passed { "PASS" } else { "FAIL" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "model={} profile={} seeds={} games={} seed_set={} status={}\n{}",
        report.algorithm_version,
        report.profile_version,
        report.seed_count,
        report.sample_size,
        report.seed_set_identity,
        if report.passed { "PASS" } else { "FAIL" },
        metrics
    )
}

fn completed_game(home_score: u16, away_score: u16, conference: bool) -> Result<Game> {
    let home = sample_team("home", "Home State", "HST", "Home")?;
    let away = sample_team("away", "Away Tech", "AWT", "Away")?;
    let mut game = Game::new(
        "demo-game",
        home,
        away,
        "Demo Stadium",
        1,
        conference,
        false,
    );
    game.start()?;
    game.home_score.add_points(Quarter::Fourth, home_score);
    game.away_score.add_points(Quarter::Fourth, away_score);
    game.complete()?;
    Ok(game)
}

fn sample_team(id: &str, name: &str, abbreviation: &str, location: &str) -> Result<Team> {
    Ok(Team::new(
        id,
        name,
        abbreviation,
        name,
        "Demo Conference",
        None,
        location,
        75,
        75,
        75,
        75,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_health_command() {
        let cli = Cli::try_parse_from(["cli", "health"]).expect("valid command");
        assert!(matches!(cli.command, Command::Health));
    }

    #[test]
    fn rng_output_is_reproducible() {
        let command = || Command::Rng {
            seed: 7,
            count: 3,
            max: 10,
        };
        assert_eq!(
            execute(command()).expect("RNG command succeeds"),
            execute(command()).expect("RNG command succeeds")
        );
    }

    #[test]
    fn game_command_reports_winner() {
        let output = execute(Command::Game {
            home_score: 24,
            away_score: 17,
            conference: true,
        })
        .expect("game command succeeds");
        assert!(output.contains("winner=Home State"));
    }

    #[test]
    fn season_command_updates_both_records() {
        let output = execute(Command::Season {
            home_score: 10,
            away_score: 20,
        })
        .expect("season command succeeds");
        assert_eq!(output, "Home State: 0-1-0\nAway Tech: 1-0-0");
    }

    #[test]
    fn parses_season_commands_and_rejects_invalid_seed() {
        assert!(matches!(
            Cli::try_parse_from(["cli", "schedule", "--seed", "7"])
                .unwrap()
                .command,
            Command::Schedule { seed: 7 }
        ));
        assert!(matches!(
            Cli::try_parse_from(["cli", "season-loop", "--seed", "7", "--full"])
                .unwrap()
                .command,
            Command::SeasonLoop {
                seed: 7,
                full: true
            }
        ));
        assert!(Cli::try_parse_from(["cli", "season-loop", "--seed", "invalid"]).is_err());
    }

    #[test]
    fn schedule_output_is_canonical_and_reproducible() {
        let first = execute(Command::Schedule { seed: 99 }).unwrap();
        let second = execute(Command::Schedule { seed: 99 }).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.lines().count(), 6);
        assert!(first.lines().next().unwrap().contains("week=1"));
        assert!(first.contains("venue=Neutral"));
    }

    #[test]
    fn weekly_and_full_season_outputs_are_reproducible() {
        for full in [false, true] {
            let command = || Command::SeasonLoop { seed: 2026, full };
            let first = execute(command()).unwrap();
            let second = execute(command()).unwrap();
            assert_eq!(first, second);
            assert!(first.contains("model=possession-v1"));
            assert!(first.contains("Standings"));
            if full {
                assert!(first.contains("state=complete next_week=none"));
                assert_eq!(first.matches("game=2026-").count(), 6);
            } else {
                assert!(first.contains("state=in-progress next_week=2"));
                assert_eq!(first.matches("game=2026-").count(), 2);
            }
        }
    }

    fn simulation_command(seed: u64, neutral: bool) -> Command {
        Command::Simulate {
            seed,
            neutral,
            home_rating: 82,
            home_offense: 84,
            home_defense: 81,
            home_special_teams: 76,
            away_rating: 75,
            away_offense: 78,
            away_defense: 73,
            away_special_teams: 72,
        }
    }

    #[test]
    fn parses_simulation_and_rejects_invalid_rating() {
        let cli = Cli::try_parse_from(["cli", "simulate", "--seed", "7", "--neutral"])
            .expect("valid command");
        assert!(matches!(
            cli.command,
            Command::Simulate { neutral: true, .. }
        ));
        assert!(Cli::try_parse_from(["cli", "simulate", "--home-rating", "101"]).is_err());
    }

    #[test]
    fn simulation_output_is_reproducible_and_complete() {
        let first = execute(simulation_command(99, false)).unwrap();
        let second = execute(simulation_command(99, false)).unwrap();
        assert_eq!(first, second);
        assert!(first.contains("Total yards"));
        assert!(first.contains("model=possession-v1"));
    }

    #[test]
    fn neutral_site_simulation_succeeds() {
        let output = execute(simulation_command(7, true)).unwrap();
        assert!(output.contains("(Final)"));
    }

    #[test]
    fn calibration_commands_return_passing_reports() {
        let text = execute(Command::Calibrate {
            seeds: 40,
            json: false,
        })
        .unwrap();
        assert!(text.contains("status=PASS"));
        let json = execute(Command::Calibrate {
            seeds: 40,
            json: true,
        })
        .unwrap();
        let report: sim_core::calibration::CalibrationReport =
            serde_json::from_str(&json).expect("machine-readable report");
        assert!(report.passed);
    }
}
