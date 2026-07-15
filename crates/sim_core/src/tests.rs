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
fn test_season_creation_and_records() {
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

    let lsu = Team::new(
        "team3",
        "LSU Tigers",
        "LSU",
        "Tigers",
        "SEC",
        Some("West".to_string()),
        "Baton Rouge, LA",
        92,
        91,
        90,
        89,
    )
    .expect("valid team");

    let teams = vec![alabama.clone(), georgia.clone(), lsu.clone()];
    let mut season = Season::new(2023, teams, 12);

    assert_eq!(season.year, 2023);
    assert_eq!(season.current_week, 1);
    assert_eq!(season.total_weeks, 12);
    assert_eq!(season.teams.len(), 3);

    // Add some games
    let mut game1 = Game::new(
        "game1",
        alabama.clone(),
        georgia.clone(),
        "Bryant-Denny Stadium, Tuscaloosa, AL",
        5,
        true,
        false,
    );

    let mut game2 = Game::new(
        "game2",
        lsu.clone(),
        alabama.clone(),
        "Tiger Stadium, Baton Rouge, LA",
        7,
        true,
        false,
    );

    // Complete the games with some scores
    game1.start().expect("scheduled game can start");
    game1.home_score.add_points(Quarter::First, 7);
    game1.away_score.add_points(Quarter::First, 3);
    game1.home_score.add_points(Quarter::Second, 10);
    game1.away_score.add_points(Quarter::Second, 7);
    game1.home_score.add_points(Quarter::Third, 0);
    game1.away_score.add_points(Quarter::Third, 7);
    game1.home_score.add_points(Quarter::Fourth, 7);
    game1.away_score.add_points(Quarter::Fourth, 0);
    game1.complete().expect("started game can complete");

    game2.start().expect("scheduled game can start");
    game2.home_score.add_points(Quarter::First, 7);
    game2.away_score.add_points(Quarter::First, 14);
    game2.home_score.add_points(Quarter::Second, 3);
    game2.away_score.add_points(Quarter::Second, 7);
    game2.home_score.add_points(Quarter::Third, 7);
    game2.away_score.add_points(Quarter::Third, 0);
    game2.home_score.add_points(Quarter::Fourth, 0);
    game2.away_score.add_points(Quarter::Fourth, 7);
    game2.complete().expect("started game can complete");

    season.add_game(game1);
    season.add_game(game2);

    // Update records
    season.update_records();

    // Check records
    let alabama_record = season.record_for_team("team1").unwrap();
    let georgia_record = season.record_for_team("team2").unwrap();
    let lsu_record = season.record_for_team("team3").unwrap();

    assert_eq!(alabama_record.wins, 2);
    assert_eq!(alabama_record.losses, 0);
    assert_eq!(alabama_record.conference_wins, 2);

    assert_eq!(georgia_record.wins, 0);
    assert_eq!(georgia_record.losses, 1);
    assert_eq!(georgia_record.conference_losses, 1);

    assert_eq!(lsu_record.wins, 0);
    assert_eq!(lsu_record.losses, 1);
    assert_eq!(lsu_record.conference_losses, 1);

    // Check standings
    let sec_standings = season.conference_standings("SEC");
    assert_eq!(sec_standings.len(), 3);
    assert_eq!(sec_standings[0].0.id, "team1"); // Alabama should be first

    // Advance week
    season.advance_week();
    assert_eq!(season.current_week, 2);
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
    let mut season = Season::new(2026, Vec::new(), 2);

    assert!(!season.is_complete());
    season.advance_week();
    assert!(!season.is_complete());
    season.advance_week();
    assert!(season.is_complete());
    assert_eq!(season.current_week, 3);
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
