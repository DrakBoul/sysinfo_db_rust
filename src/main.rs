use std::{alloc::System, error::Error, io, sync::{Arc, Mutex}, thread, time::Duration};
use chrono::prelude::*;
use rusqlite::{Params, Connection, Result};
use sysinfo::{
    Components, Disks, System as SystemData,
};


fn main() {

    //establish connection to db and handle errors
    let conn = match Connection::open("./data/sysinfo.db") {
        Ok(conn) => conn,
        Err(e) => {
            println!("Connection failed. Make sure the db exists and the path is correct");
            println!("{}", e);
            return;
        }
    };

    // Create the database schema
    create_schema(&conn);


    let mut sys = SystemData::new_all();

    write_sysdata(&mut sys, &conn);

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
            3 => view_records(&conn),
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

fn view_records_menu(input: &mut String) -> u8 {

    println!("Please select which type of records to view:");
    println!("1.    System Data");
    println!("2.    Components");
    println!("3.    Ram and Swap");
    println!("4.    Disks");
    println!("5.    Go back");

    io::stdin()
            .read_line( input)
            .expect("Failed to read input.");

        let input: u8 = match input.trim().parse() {
            Ok(n) => n,

            Err(_) => {
                println!("Invalid input. Please enter a number in the range 1-5.");
                0
            }
        }; 
        // return our input from user back to the view records function
        input
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
    println!("Stopped recording...");
    
}

fn view_records(conn: &Connection) {

    loop {
        let mut input = String::new();
        let input = view_records_menu(&mut input);

        match input {
            1 => {view_sys_records(conn);},
            // 2 => view_component_records(conn),
            // 3 => view_ram_records(conn),
            // 4 => view_disk_records(conn),
            5 => return,
            _ => {
                println!("Invalid input. Please enter a number 1-5.");
                continue;
            }
        }
    }
    

}

fn live_data_feed() {
    println!("Starting live data feed...");
    println!("Press 'q' then enter to return to main menu.");
    loop {
        
    }
}

fn create_schema(conn: &Connection) {

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS component (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                label TEXT NOT NULL,
                temp INTEGER NOT NULL
                )",
        ()
    ) {
        Ok(_) => {},
        Err(e) => {
            println!("Creating table 'component' failed...");
            println!("{}", e);
            return;
        }
    }

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS disk (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                name TEXT NOT NULL,
                total INTEGER NOT NULL,
                available INTEGER NOT NULL
                )",
        ()
    ) {
        Ok(_) => {},
        Err(e) => {
            println!("Creating table 'disk' failed");
            println!("{}", e);
        }
    }

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS ram (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                label TEXT NOT NULL,
                temp INTEGER NOT NULL
                )",
        ()
    ) {
        Ok(_) => {},
        Err(e) => {
            println!("Creating table 'ram' failed");
            println!("{}", e);
        }
    }

    match conn.execute(
        "CREATE TABLE IF NOT EXISTS sys (
                id INTEGER PRIMARY KEY,
                os TEXT NOT NULL,
                osversion TEXT NOT NULL,
                hostname TEXT NOT NULL
                )",
        ()
    ) {
        Ok(_) => {},
        Err(e) => {
            println!("Creating 'sys' table failed");
            println!("{}", e);
        }
    }
}

fn write_sysdata(sys: &mut SystemData, conn: &Connection) {
    // TODO: This should only write once

    sys.refresh_all();

    match conn.execute(
        "INSERT INTO sys 
        (os, osversion, hostname) VALUES (?1, ?2, ?3)", 
        (SystemData::name().unwrap(), SystemData::os_version().unwrap(), SystemData::host_name().unwrap())
    ) 
        {
            Ok(_) => {},
            Err(e) => {
                println!("Error writing system data.");
                println!("{}", e);
                return;
            }       
        }
}

fn view_sys_records(conn: &Connection) -> Result<()>{

    println!("Viewing System records...");

    let mut stmt = conn.prepare(
        "SELECT os, osversion, hostname FROM sys"
    )?;

    let sysdata_iter = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    })?;

    for sysdata in sysdata_iter {
        println!("Name: {} , Version: {} , hostname: {}", sysdata.as_ref().unwrap().0, sysdata.as_ref().unwrap().1, sysdata.as_ref().unwrap().2);
    }
    Ok(())
}

fn view_component_records(conn: &Connection ) {
   println!("Viewing Component records...");
 
}

fn view_ram_records(conn: &Connection ) {
    println!("Viewing RAM records...");
    
}

fn view_disk_records(conn: &Connection ) {
    println!("Viewing Disk records...");

}

