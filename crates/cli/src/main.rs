//! cli: Command-line interface entrypoint for the simulator workspace.
//!
//! Commands in this binary provide a lightweight harness for exercising features
//! as they are added to the domain and persistence crates.

use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use sim_core::calibration::run_calibration;
use sim_core::game::{Game, Quarter};
use sim_core::rng::SimRng;
use sim_core::season::Season;
use sim_core::simulation::{
    simulate_game, Matchup, MatchupModifiers, SimulationConfig, SimulationResult, Venue,
};
use sim_core::team::Team;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

mod roadmap;

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
    /// Validate and render the version-controlled product roadmap
    Roadmap {
        /// Repository root containing roadmap.yaml and openspec/
        #[arg(long, default_value = ".")]
        root: std::path::PathBuf,
        #[command(subcommand)]
        command: RoadmapCommand,
    },
}

#[derive(Subcommand, Debug)]
enum RoadmapCommand {
    /// Validate roadmap structure, dependencies, lifecycle, and evidence
    Validate,
    /// Regenerate ROADMAP.md from roadmap.yaml
    Render,
    /// Verify ROADMAP.md is the current canonical rendering without writing
    Check,
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
            let game = completed_game(home_score, away_score, true)?;
            let home_id = game.home_team.id.clone();
            let away_id = game.away_team.id.clone();
            let mut season = Season::new(
                2026,
                vec![game.home_team.clone(), game.away_team.clone()],
                12,
            );
            season.add_game(game);
            season.update_records();
            let home = season
                .record_for_team(&home_id)
                .ok_or_else(|| anyhow::anyhow!("home record was not created"))?;
            let away = season
                .record_for_team(&away_id)
                .ok_or_else(|| anyhow::anyhow!("away record was not created"))?;
            Ok(format!("Home State: {home}\nAway Tech: {away}"))
        }
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
        Command::Roadmap { root, command } => match command {
            RoadmapCommand::Validate => roadmap::validate_at(&root),
            RoadmapCommand::Render => roadmap::render_at(&root),
            RoadmapCommand::Check => roadmap::check_at(&root),
        },
    }
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

    #[test]
    fn roadmap_commands_parse() {
        for action in ["validate", "render", "check"] {
            let cli = Cli::try_parse_from(["cli", "roadmap", "--root", ".", action])
                .expect("valid roadmap command");
            assert!(matches!(cli.command, Command::Roadmap { .. }));
        }
    }

    #[test]
    fn roadmap_commands_execute_against_repository_fixture() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("roadmap.yaml"),
            "schema_version: 1\nitems:\n  - id: TEST-01\n    sequence: 1\n    title: Test roadmap\n    theme: test\n    status: exploring\n    outcome: The roadmap works.\n    exclusions: [Delivery]\n",
        )
        .unwrap();
        let command = |action| Command::Roadmap {
            root: root.path().to_path_buf(),
            command: action,
        };
        assert!(execute(command(RoadmapCommand::Validate))
            .unwrap()
            .contains("1 items"));
        execute(command(RoadmapCommand::Render)).unwrap();
        assert!(execute(command(RoadmapCommand::Check))
            .unwrap()
            .contains("current"));
        std::fs::write(root.path().join("ROADMAP.md"), "stale\n").unwrap();
        assert!(execute(command(RoadmapCommand::Check)).is_err());
    }
}
