use std::io;

fn main() {
    println!("Welcome to the sysinfo database!");
    // Loop to handle user input
    loop {
        start_menu();
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input.");
        
        let input: u8 = match input.trim().parse() {
            Ok(n) => n,
            //TODO: add more useful error information by including an additional match statement for different types of errors
            Err(_) => {
                println!("Invalid input. Please enter a number in the range 1-5.");
                continue;
            }
        };

        match input {
            1 => start_recording(),
            2 => stop_recording(),
            3 => view_records(),
            4 => live_data_feed(),
            5 => {
                println!("Quitting Program...");
                break;

            }
            _ => println!("Invalid input. Please enter a number in the range 1-5.")
        };

    }
}

fn start_menu() {
    println!("Please select one of the options below by typing the respective number and pressing the 'Enter' key.");
    println!("1.    Start recording");
    println!("2.    Stop recording");
    println!("3.    View records");
    println!("4.    Live data feed");
    println!("5.    Quit Program");
}

// TODO: Write these 4 functions for the main logic of the program
fn start_recording() {

    println!("Starting recording... We will keep recording data for you until you stop recording");
    println!("enter 'q' at any time to return to the main menu");

    loop {
        let mut input: String = String::new(); 
        io::stdin()
            .read_line(&mut input)
            .expect("Reading failed");

        let input: char = match input.trim().parse() {
            Ok(c) => c,
            Err(_) => {
                println!("Invlaid Input. Please enter 'q' to exit to the main menu.");
                continue;
            }
        };

        match input {
            'q' => break,
            _ => {
                println!("Invlaid Input. Please enter 'q' to exit to the main menu.");
                continue;
            }
        }
    }
}

fn stop_recording() {
    println!("Stopping recording...");
}

fn view_records() {
    println!("Viewing records...");
}

fn live_data_feed() {
    println!("Starting live data feed...");
}