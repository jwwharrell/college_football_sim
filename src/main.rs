extern crate sqlite;

use sqlite::{Connection, Result};

mod cli;
mod dao;

fn main() -> Result<()> {
    let connection = sqlite::open("db/college_football_simulator.db")?;

    create_tables(&connection)?;
    seed_database(&connection)?;

    println!("Database setup complete.");

    // game logic will go here
    cli::start_menu();
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS Schools (
            SchoolID INTEGER PRIMARY KEY,
            Name TEXT NOT NULL,
            Division TEXT NOT NULL CHECK (Division IN ('1fbs', '1fcs', '2', '3')),
            Budget REAL NOT NULL
        );
        CREATE TABLE IF NOT EXISTS Coaches (
            CoachID INTEGER PRIMARY KEY,
            Name TEXT NOT NULL,
            OverallRating INTEGER NOT NULL,
            OffensiveSkill INTEGER NOT NULL,
            DefensiveSkill INTEGER NOT NULL,
            RecruitingAbility INTEGER NOT NULL,
            GameStrategy INTEGER NOT NULL,
            DevelopmentSkill INTEGER NOT NULL,
            SchoolID INTEGER,
            FOREIGN KEY (SchoolID) REFERENCES Schools (SchoolID)
        );
        CREATE TABLE IF NOT EXISTS Athletes (
            AthleteID INTEGER PRIMARY KEY,
            Name TEXT NOT NULL,
            Position TEXT NOT NULL CHECK (Position IN ('Quarterback', 'Running Back', 'Wide Receiver', 'Defensive Line', 'Linebacker', 'Offensive Line', 'Tight End', 'Cornerback', 'Safety', 'Kicker', 'Punter')),
            OverallRating INTEGER NOT NULL,
            Strength INTEGER NOT NULL,
            Speed INTEGER NOT NULL,
            Agility INTEGER NOT NULL,
            Stamina INTEGER NOT NULL,
            InjuryProneness INTEGER NOT NULL,
            SchoolID INTEGER,
            CoachID INTEGER,
            FOREIGN KEY (SchoolID) REFERENCES Schools (SchoolID),
            FOREIGN KEY (CoachID) REFERENCES Coaches (CoachID)
        );
        "
    )?;
    Ok(())
}

fn seed_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        INSERT INTO Schools (Name, Division, Budget) VALUES
        ('North Carolina State', '1fbs', 10000000),
        ('Chowan', '2', 400000);

        INSERT INTO Coaches (Name, OverallRating, OffensiveSkill, DefensiveSkill, RecruitingAbility, GameStrategy, DevelopmentSkill, SchoolID) VALUES
        ('Coach A', 80, 70, 65, 60, 75, 70, 1),
        ('Coach B', 75, 65, 70, 55, 65, 65, 2);

        INSERT INTO Athletes (Name, Position, OverallRating, Strength, Speed, Agility, Stamina, InjuryProneness, SchoolID, CoachID) VALUES
        ('Player A1', 'Quarterback', 85, 70, 80, 75, 60, 10, 1, 1),
        ('Player A2', 'Running Back', 80, 65, 85, 70, 55, 20, 1, 1),
        ('Player B1', 'Wide Receiver', 78, 60, 85, 80, 50, 15, 2, 2),
        ('Player B2', 'Linebacker', 82, 75, 70, 65, 60, 5, 2, 2);
        "
    )?;
    Ok(())
}
