use inquire::{InquireError, Select};

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
                    Err(e) => println!("Error fetching school: {}", e),
                }
            }
            println!("You selected {}!", choice)
        }
        Err(_) => println!("There was an error, please try again"),
    }
}
