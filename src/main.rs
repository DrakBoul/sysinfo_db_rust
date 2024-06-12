// use std::io;

fn main() {
        println!("Welcome to the sysinfo database!");
        start_menu();
        // let mut input = String::new();

        // io::stdin()
        //     .read_line(&mut input)
        //     .expect("Failed to read input.");
        
        // let input: u8 = match input.trim().parse() {

        //     Ok(n) => n,

        //     Err(e) => {

        //         println!("{}", e);
        //         0
        //     }
        // };
        
}

fn start_menu() {
    println!("Please select one of the options below by typing the respective number and pressing the 'Enter' key.");
    println!("1.    Start recording");
    println!("2.    Stop recording");
    println!("3.    View records");
    println!("4.    Live data feed");
    println!("5.    Quit Program");
}

// fn start_recording_menu() {
//     println!("We started recording your data, we will keep recording until you choose stop recording in the main menu.");
//     println!("Press 'q' to go back to the main menu at anytime");
// }