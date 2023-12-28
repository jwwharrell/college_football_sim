use std::{fmt, result};

extern crate sqlite;

use sqlite::{Row, State};

#[derive(Debug)]
pub enum MyError {
    NotFound,
    SqliteError(sqlite::Error),
}

impl From<sqlite::Error> for MyError {
    fn from(err: sqlite::Error) -> MyError {
        MyError::SqliteError(err)
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::NotFound => write!(f, "School not found"),
            MyError::SqliteError(e) => write!(f, "SQLite Error: {}", e),
        }
    }
}

pub struct School {
    id: i64,
    name: String,
    division: String,
    budget: f64,
}

impl fmt::Display for School {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}, name: {}, division: {}, budget: {}",
            self.id, self.name, self.division, self.budget
        )
    }
}

pub struct SchoolDAO {
    conn: sqlite::Connection,
}

impl SchoolDAO {
    pub fn new(conn: sqlite::Connection) -> Self {
        SchoolDAO { conn }
    }

    pub fn get_by_id(&self, school_id: i64) -> Result<School, MyError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM Schools WHERE SchoolID = ?")?;

        stmt.bind((1, school_id))?;

        if let State::Row = stmt.next()? {
            let school = School {
                id: stmt.read::<i64, _>("SchoolID")?,
                name: stmt.read::<String, _>("Name")?,
                division: stmt.read::<String, _>("Division")?,
                budget: stmt.read::<f64, _>("Budget")?,
            };
            Ok(school)
        } else {
            Err(MyError::NotFound)
        }
    }

    pub fn list(&self) -> Result<Vec<(i64, String)>, sqlite::Error> {
        let mut stmt = self.conn.prepare("SELECT SchoolID, Name FROM Schools")?;

        let mut results = Vec::new();
        while let State::Row = stmt.next()? {
            let id = stmt.read::<i64, _>(0)?;
            let name = stmt.read::<String, _>(1)?;

            results.push((id, name));
        }

        Ok(results)
    }
}
