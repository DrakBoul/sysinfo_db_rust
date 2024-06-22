use std::{alloc::System, error::Error, fmt::{self, Formatter}, io, result, sync::{Arc, Mutex}, thread, time::Duration};
use chrono::prelude::*;
use rusqlite::{Params, Connection, Result, Row};
use sysinfo::{Components, Disk, Disks, System as SystemData};


trait Record: Sized + fmt::Display {
    fn write_to_db(&self, conn: &Connection) -> Result<()>;
    fn query() -> &'static str;
    fn from_row(row: &Row) -> Result<Self>;
    
}

struct SysRecord {
    os: String,
    osversion: String,
    hostname: String
}

impl fmt::Display for SysRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OS: {} , Version: {} , Hostname: {}", self.os, self.osversion, self.hostname)
    }
}

impl Record for SysRecord {

    fn write_to_db(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO sys (os, osversion, hostname) VALUES (?1, ?2, ?3)", 
        (&self.os, &self.osversion, &self.hostname))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT os, osversion, hostname FROM sys"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(SysRecord {
            os: row.get(0)?,
            osversion: row.get(1)?,
            hostname: row.get(2)?,
        })
    }
}


struct ComponentRecord {
    timestamp: String,
    label: String,
    temp: i32
}

impl fmt::Display for ComponentRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Label: {} , Temperature: {}", self.timestamp, self.label, self.temp)
    }
}

impl Record for ComponentRecord {

    fn write_to_db(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO component (timestamp, label, temp) VALUES (?1, ?2, ?3)", 
        (&self.timestamp, &self.label, &self.temp))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT timestamp, label, temp FROM components"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(ComponentRecord {
            timestamp: row.get(0)?,
            label: row.get(1)?,
            temp: row.get(2)?,
        })
    }
        
}


struct DiskRecord {
    timestamp: String,
    name: String,
    total: u64,
    available: u64
}

impl fmt::Display for DiskRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Name: {} , Total: {} , Available: {}", self.timestamp, self.name, self.total, self.available)
    }
}

impl Record for DiskRecord {

    fn write_to_db(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO component (timestamp, name, total, available) VALUES (?1, ?2, ?3, ?4)", 
        (&self.timestamp, &self.name, &self.total, &self.available))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT timestamp, name, total, available FROM disk"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(DiskRecord {
            timestamp: row.get(0)?,
            name: row.get(1)?,
            total: row.get(2)?,
            available: row.get(3)?,
        })
    }
}

struct RAMRecord {
    timestamp: String,
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64
}

impl fmt::Display for RAMRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Total Memory: {} , Used Memory: {} , Total Swap: {} , Used Swap: {}", 
        self.timestamp, self.total_memory, self.used_memory, self.total_swap, self.used_swap)
    }
}

impl Record for RAMRecord {

    fn write_to_db(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO component (timestamp, total_memory, used_memory, total_swap, used_swap) VALUES (?1, ?2, ?3, ?4, ?5)", 
        (&self.timestamp, &self.total_memory, &self.used_memory, &self.total_swap, &self.used_swap))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT timestamp, total_memory, used_memory, total_swap, used_swap FROM ram"
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(RAMRecord {
            timestamp: row.get(0)?,
            total_memory: row.get(1)?,
            used_memory: row.get(2)?,
            total_swap: row.get(3)?,
            used_swap: row.get(4)?,
        })
    }
}

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
            1 => {let _ = query_db_all::<SysRecord>(conn);},
            2 => {let _ = query_db_all::<ComponentRecord>(conn);}
            3 => {let _ = query_db_all::<RAMRecord>(conn);}
            4 => {let _ = query_db_all::<DiskRecord>(conn);}
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
                total_memory INTEGER NOT NULL,
                used_memory INTEGER NOT NULL,
                total_swap INTEGER NOT NULL,
                used_swap INTEGER NOT NULL
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
    let sys_record = SysRecord {
        os: SystemData::name().unwrap(),
        osversion: SystemData::os_version().unwrap(), 
        hostname: SystemData::host_name().unwrap(),
    };
    // Does not currently use result returned
    let _ = sys_record.write_to_db(conn);
    
}


fn query_db_all<T: Record>(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(T::query())?;
    let record_iter = stmt.query_map([], |row| T::from_row(row))?;
    
    for record in record_iter {
        println!("{}", record?);
    }

    Ok(())
}

