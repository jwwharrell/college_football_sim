use inquire::{InquireError, Select, Text};

use crate::dao::school::SchoolDAO;

pub fn start_menu() {
    let options = vec!["List Schools", "Search for School", "Simulate Game", "Exit"];

    let ans: Result<&str, InquireError> =
        Select::new("Welcome to College Football Simulator!", options).prompt();

    match ans {
        Ok(choice) => {
            if choice == "List Schools" {
                let conn = sqlite::open("db/college_football_simulator.db").unwrap();
                let dao = SchoolDAO::new(conn);
                match dao.list() {
                    Ok(schools) => {
                        for tuple in schools {
                            println!("{:?}", tuple)
                        }
                    }
                    Err(e) => println!("Error listing schools: {}", e),
                }
            }
            if choice == "Search for School" {
                let school_id_prompt =
                    Text::new("Enter the ID of the school you would like to view:").prompt();

                match school_id_prompt {
                    Ok(school_id) => {
                        let conn = sqlite::open("db/college_football_simulator.db").unwrap();
                        let dao = SchoolDAO::new(conn);
                        let parsed_school_id = school_id.parse::<i64>().unwrap();
                        match dao.get_by_id(parsed_school_id) {
                            Ok(school) => {
                                println!("School Detail: {}", school)
                            }
                            Err(e) => println!("Error fetching school: {}", e),
                        }
                    }
                    Err(_) => println!("There was an error with the given input."),
                }
            }
            println!("You selected {}!", choice)
        }
        Err(_) => println!("There was an error, please try again"),
    }
}
