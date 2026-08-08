//! Tests for the sim_core crate.
//!
//! This module contains integration tests for the core domain types and RNG façade.

use crate::game::{Game, GameStatus, Quarter};
use crate::rng::SimRng;
use crate::season::Season;
use crate::team::{Team, TeamBuilder};

#[test]
fn test_team_creation() {
    let team = Team::new(
        "team1",
        "Alabama Crimson Tide",
        "ALA",
        "Crimson Tide",
        "SEC",
        Some("West".to_string()),
        "Tuscaloosa, AL",
        95,
        92,
        94,
        90,
    )
    .expect("valid team");

    assert_eq!(team.id, "team1");
    assert_eq!(team.name, "Alabama Crimson Tide");
    assert_eq!(team.abbreviation, "ALA");
    assert_eq!(team.conference, "SEC");
    assert_eq!(team.division, Some("West".to_string()));
    assert_eq!(team.full_name(), "Tuscaloosa, AL Crimson Tide");
}

#[test]
fn test_team_builder() {
    let team = TeamBuilder::new()
        .id("team2")
        .name("Georgia Bulldogs")
        .abbreviation("UGA")
        .mascot("Bulldogs")
        .conference("SEC")
        .division("East")
        .location("Athens, GA")
        .rating(94)
        .offense_rating(93)
        .defense_rating(95)
        .special_teams_rating(92)
        .build()
        .expect("valid team");

    assert_eq!(team.id, "team2");
    assert_eq!(team.name, "Georgia Bulldogs");
    assert_eq!(team.abbreviation, "UGA");
    assert_eq!(team.conference, "SEC");
    assert_eq!(team.division, Some("East".to_string()));
}

#[test]
fn test_game_creation_and_scoring() {
    let alabama = Team::new(
        "team1",
        "Alabama Crimson Tide",
        "ALA",
        "Crimson Tide",
        "SEC",
        Some("West".to_string()),
        "Tuscaloosa, AL",
        95,
        92,
        94,
        90,
    )
    .expect("valid team");

    let georgia = Team::new(
        "team2",
        "Georgia Bulldogs",
        "UGA",
        "Bulldogs",
        "SEC",
        Some("East".to_string()),
        "Athens, GA",
        94,
        93,
        95,
        92,
    )
    .expect("valid team");

    let mut game = Game::new(
        "game1",
        alabama.clone(),
        georgia.clone(),
        "Bryant-Denny Stadium, Tuscaloosa, AL",
        5,
        true,
        false,
    );

    assert_eq!(game.status, GameStatus::Scheduled);
    assert!(game.is_conference_game);
    assert!(!game.is_neutral_site);

    // Start the game
    game.start().expect("scheduled game can start");
    assert_eq!(game.status, GameStatus::InProgress);
    assert_eq!(game.current_quarter, Some(Quarter::First));

    // Add some scoring
    game.home_score.add_points(Quarter::First, 7);
    game.away_score.add_points(Quarter::First, 3);
    game.home_score.add_points(Quarter::Second, 10);
    game.away_score.add_points(Quarter::Second, 7);
    game.home_score.add_points(Quarter::Third, 0);
    game.away_score.add_points(Quarter::Third, 7);
    game.home_score.add_points(Quarter::Fourth, 7);
    game.away_score.add_points(Quarter::Fourth, 7);

    // Complete the game
    game.complete().expect("started game can complete");
    assert_eq!(game.status, GameStatus::Completed);
    assert_eq!(game.home_score.total, 24);
    assert_eq!(game.away_score.total, 24);
    assert!(game.is_tie());
    assert_eq!(game.winner(), None);
    assert_eq!(game.loser(), None);
}

#[test]
fn test_rng_determinism() {
    let seed = 42;
    let mut rng1 = SimRng::new(seed);
    let mut rng2 = SimRng::new(seed);

    // Both RNGs should produce the same sequence
    for _ in 0..10 {
        assert_eq!(rng1.int(100), rng2.int(100));
        assert_eq!(rng1.float(), rng2.float());
    }

    // Different seeds should produce different results
    let mut rng3 = SimRng::new(seed + 1);
    let mut all_same = true;
    for _ in 0..10 {
        if rng1.int(1000) != rng3.int(1000) {
            all_same = false;
            break;
        }
    }
    assert!(
        !all_same,
        "Different seeds should produce different sequences"
    );
}

#[test]
fn season_becomes_complete_after_advancing_past_final_week() {
    let mut season = Season::new(2026, Vec::new(), 2, Vec::new()).unwrap();

    assert!(!season.is_complete());
    season
        .advance_week(42, &crate::simulation::SimulationConfig::default())
        .unwrap();
    assert!(!season.is_complete());
    season
        .advance_week(42, &crate::simulation::SimulationConfig::default())
        .unwrap();
    assert!(season.is_complete());
    assert_eq!(season.current_week(), None);
}

#[test]
fn team_creation_rejects_out_of_range_rating() {
    let result = Team::new(
        "team1",
        "Test Team",
        "TST",
        "Testers",
        "Test",
        None,
        "Testville",
        101,
        50,
        50,
        50,
    );

    assert!(result.is_err());
}

#[test]
fn game_rejects_invalid_state_transitions() {
    let home = Team::new(
        "home",
        "Home Team",
        "HOM",
        "Homes",
        "Test",
        None,
        "Home",
        50,
        50,
        50,
        50,
    )
    .expect("valid team");
    let away = Team::new(
        "away",
        "Away Team",
        "AWY",
        "Aways",
        "Test",
        None,
        "Away",
        50,
        50,
        50,
        50,
    )
    .expect("valid team");
    let mut game = Game::new("game", home, away, "Stadium", 1, true, false);

    assert!(game.complete().is_err());
    game.start().expect("scheduled game can start");
    assert!(game.start().is_err());
    game.complete().expect("started game can complete");
    assert!(game.cancel().is_err());
}
