use inquire::{InquireError, Select};

pub fn start_menu() {
    let options: Vec<&str> = vec!["List Schools", "Search for School", "Simulate Game", "Exit"];

    let ans: Result<&str, InquireError> = Select::new("Welcome to College Football Simulator!", options).prompt();
    
    match ans {
        Ok(choice) => println!("You selected {}!", choice),
        Err(_) => println!("There was an error, please try again"),
    }
}
