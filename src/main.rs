use std::{fmt, io, sync::{mpsc::Sender, Arc, Mutex}, thread, time::Duration};
use chrono::prelude::*;
use rusqlite::{Connection, Result, Row};
use sysinfo::{Components, Disks, System as SystemData};
use regex::Regex;
use std::sync::mpsc;

trait Record: Sized + fmt::Display {
    fn write_to_db(&self, conn: Arc<Mutex<Connection>>) -> Result<()>;
    fn query() -> &'static str;
    fn query_by_dt(start_dt: String, end_dt: String) -> String;
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

    fn write_to_db(&self, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sys (os, osversion, hostname) VALUES (?1, ?2, ?3)", 
        (&self.os, &self.osversion, &self.hostname))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT os, osversion, hostname FROM sys"
    }

    fn query_by_dt(start_dt: String, end_dt: String) -> String {
        // no functionality currently needed for querying system records by datetime
        let _ = start_dt;
        let _ = end_dt;
        "SELECT os, osversion, hostname FROM sys".to_string()
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
    datetime: String,
    label: String,
    temp: f32
}

impl fmt::Display for ComponentRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Label: {} , Temperature: {}", self.datetime, self.label, self.temp)
    }
}

impl Record for ComponentRecord {

    fn write_to_db(&self, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO component (datetime, label, temp) VALUES (?1, ?2, ?3)", 
        (&self.datetime, &self.label, &self.temp))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT datetime, label, temp FROM component"
    }

    fn query_by_dt(start_dt: String, end_dt: String) -> String {
        // no functionality currently needed for querying system records by datetime
        format!("SELECT datetime, label, temp FROM component WHERE datetime BETWEEN '{}' AND '{}'", start_dt, end_dt)
        
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(ComponentRecord {
            datetime: row.get(0)?,
            label: row.get(1)?,
            temp: row.get(2)?,
        })
    }
        
}


struct DiskRecord {
    datetime: String,
    name: String,
    total: u64,
    available: u64
}

impl fmt::Display for DiskRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Name: {} , Total: {} , Available: {}", self.datetime, self.name, self.total, self.available)
    }
}

impl Record for DiskRecord {

    fn write_to_db(&self, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO disk (datetime, name, total, available) VALUES (?1, ?2, ?3, ?4)", 
        (&self.datetime, &self.name, &self.total, &self.available))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT datetime, name, total, available FROM disk"
    }
    
    fn query_by_dt(start_dt: String, end_dt: String) -> String {
        format!("SELECT datetime, name, total, available FROM disk WHERE datetime BETWEEN '{}' AND '{}'", start_dt, end_dt)
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(DiskRecord {
            datetime: row.get(0)?,
            name: row.get(1)?,
            total: row.get(2)?,
            available: row.get(3)?,
        })
    }
}

struct RAMRecord {
    datetime: String,
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64
}

impl fmt::Display for RAMRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {} , Total Memory: {} , Used Memory: {} , Total Swap: {} , Used Swap: {}", 
        self.datetime, self.total_memory, self.used_memory, self.total_swap, self.used_swap)
    }
}

impl Record for RAMRecord {

    fn write_to_db(&self, conn: Arc<Mutex<Connection>>) -> Result<()> {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ram (datetime, total_memory, used_memory, total_swap, used_swap) VALUES (?1, ?2, ?3, ?4, ?5)", 
        (&self.datetime, &self.total_memory, &self.used_memory, &self.total_swap, &self.used_swap))?;
        Ok(())
    }

    fn query() -> &'static str {
        "SELECT datetime, total_memory, used_memory, total_swap, used_swap FROM ram"
    }

    fn query_by_dt(start_dt: String, end_dt: String) -> String {
        format!("SELECT datetime, total_memory, used_memory, total_swap, used_swap FROM ram WHERE datetime BETWEEN {} AND {}", start_dt, end_dt)
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(RAMRecord {
            datetime: row.get(0)?,
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

    let conn = Arc::new(Mutex::new(conn));

    let (tx, rx) = mpsc::channel();

    // Start record function to listen when to record initalize by telling it not to record
    tx.send(0).unwrap();

    let conn_thread = conn.clone();
    thread::spawn(move || {
        create_schema(conn_thread.clone());
        let mut sys = SystemData::new_all();
        write_sysdata(&mut sys, conn_thread.clone());
    
        let mut recording = false;
        let mut p = false;

        loop {
            match rx.try_recv() {
                Ok(incoming) => {
                    if incoming == 0 {
                        recording = false;
                        p = false;
                        continue; 
                    } else if incoming == 1 {
                        recording = true; 
                        p = false;
                    } else {
                        recording = true;
                        p = true;
                    }
                }
                Err(_) => {} 
            }

            if recording {
                write_all_records(&mut sys, conn_thread.clone(), p); // Example recording action
                thread::sleep(Duration::from_secs(10));
            }
        }
    });

    println!("Welcome to the sysinfo database!");
    // Loop to handle user input
    loop {
        start_menu();
        let input = read_string("");
        
        let input: u8 = match input.trim().parse() {
            Ok(n) => n,
            //TODO: add more useful error information by including an additional match statement for different types of errors
            Err(_) => {
                println!("Invalid input. Please enter a number in the range 1-5.");
                continue;
            }
        };

        match input {
            1 => start_recording(tx.clone()),
            2 => stop_recording(tx.clone()),
            3 => view_records(conn.clone()),
            4 => live_data_feed(tx.clone()),
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

fn view_records_menu() -> u8 {

    println!("Please select which type of records to view:");
    println!("1.    System Data");
    println!("2.    Components");
    println!("3.    Ram and Swap");
    println!("4.    Disks");
    println!("5.    Go back");
    
    let input = read_string("");
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
fn start_recording(tx: Sender<u8>) {
    // Send message to start recording
    if let Err(err) = tx.send(1) {
        eprintln!("Failed to send message: {}", err);
        return;
    }
    println!("Starting recording... We will keep recording data for you until you stop recording");
    println!("enter 'q' at any time to return to the main menu");

    loop {
        let input = read_string("");

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

fn stop_recording(tx: Sender<u8>) {
    tx.send(0).unwrap();
    println!("Stopped recording...");
    
}

fn view_records(conn: Arc<Mutex<Connection>>) {

    loop {
        let input = view_records_menu();
        let conn_clone = conn.clone();
        match input {
            1 => {let _ = print_records(query_db_all::<SysRecord>(conn_clone));},
            2 => {query_choice::<ComponentRecord>(conn_clone)}
            3 => {query_choice::<RAMRecord>(conn_clone)}
            4 => {query_choice::<DiskRecord>(conn_clone)}
            5 => return,
            _ => {
                println!("Invalid input. Please enter a number 1-5.");
                continue;
            }
        }

    }
    

}

fn live_data_feed(tx: Sender<u8>) {
    tx.send(2).unwrap();
    println!("Starting live data feed...");
    println!("Press 'q' then enter to return to main menu.");
    loop {
        let input = read_string("");

        let input: char = match input.trim().parse() {
            Ok(c) => c,
            Err(_) => {
                println!("Invlaid Input. Please enter 'q' to exit to the main menu.");
                continue;
            }
        };

        match input {
            'q' => {
                tx.send(1).unwrap();
                break
            },
            _ => {
                println!("Invlaid Input. Please enter 'q' to exit to the main menu.");
                continue;
            }
        }
    }
}

fn create_schema(conn: Arc<Mutex<Connection>>) {
    let conn = conn.lock().unwrap();
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS component (
                id INTEGER PRIMARY KEY,
                datetime DATETIME NOT NULL,
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
                datetime DATETIME NOT NULL,
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
                datetime DATETIME NOT NULL,
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

fn write_sysdata(sys: &mut SystemData, conn: Arc<Mutex<Connection>>) {
    // Refresh system data
    sys.refresh_all();
    // Create a new SysRecord with current system information
    let sys_record = SysRecord{
        os: SystemData::name().unwrap(),
        osversion: SystemData::os_version().unwrap(),
        hostname: SystemData::host_name().unwrap(),
    };
    // Query existing SysRecords from the database
    let old_records_result = query_db_all::<SysRecord>(conn.clone());
    match old_records_result {
        Ok(old_records) => {
            if old_records.is_empty() {
                // If there are no old records, write the current sys_record to the database
                match sys_record.write_to_db(conn.clone()) {
                    Ok(_) => println!("System data written successfully."),
                    Err(e) => println!("Error occurred while writing system data: {}", e),
                }
            } else {
                // Check if the current hostname already exists in the old records
                let mut hostname_exists = false;
                for record in &old_records {
                    if record.hostname == sys_record.hostname {
                        hostname_exists = true;
                        println!("System data not written: record for '{}' already exists.", sys_record.hostname);
                        break;
                    }
                }

                // If the hostname doesn't exist, write the sys_record to the database
                if !hostname_exists {
                    match sys_record.write_to_db(conn.clone()) {
                        Ok(_) => println!("System data written successfully."),
                        Err(e) => println!("Error occurred while writing system data: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            println!("Error retrieving old records: {}", e);
        }
    }
}


fn query_db_all<T>(conn: Arc<Mutex<Connection>>) -> Result<Vec<T>>
where
    T: Record {
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(T::query())?;
    let record_iter = stmt.query_map([], |row| T::from_row(row))?;

    let mut records = Vec::new();
    for record in record_iter {
        records.push(record?);
    }

    Ok(records)
}


fn print_records<T>(records: Result<Vec<T>>) -> Result<()>
where
    T: Record + std::fmt::Display {
    match records {
        Ok(records) => {
            for record in records {
                println!("{}", record);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn query_by_dt<T>(conn: Arc<Mutex<Connection>>, start_dt: String, end_dt: String) -> Result<Vec<T>>
where 
    T: Record {

    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare(&T::query_by_dt(start_dt, end_dt))?;
    let record_iter = match stmt.query_map([], |row| T::from_row(row)) {
        Ok(record_iter) => record_iter,
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            return Err(e);
        }
    };
    
    let mut records = Vec::new();
    for record in record_iter {
        records.push(record?);
    }

    Ok(records)


}

fn query_choice<T>(conn: Arc<Mutex<Connection>>) 
where 
    T: Record {
    loop {
        println!("Choose how to query Records:");
        println!("1.    All Records");
        println!("2.    By Date Time");
        println!("3.    Go Back");
        let input = read_string("");
        let input: u8 = match input.trim().parse() {
            Ok(i) => i,
            Err(_) => {
                println!("Invlaid Input. Please enter a number.");
                continue;
            }
        };
        match input {
            1 => {
                let _ = print_records(query_db_all::<T>(conn.clone()));
            },
            2 => {
                let dates = get_datetime_range();
                let start_dt = &dates[0];
                let end_dt = &dates[1];
                let _ = print_records::<T>(query_by_dt::<T>(conn.clone(), start_dt.to_string(), end_dt.to_string()));

            },
            3 => {
                break;
            }
            _ => {
                println!("Please enter one of the options given.");
                continue;
            }
        }
    }


}


// TODO: Refactor all the times we read input to use this function.
fn read_string(prompt: &str) -> String {

    println!("{}", prompt);
    let mut input: String = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read.");
    input
}

fn parse_datetime_range(dt_range: String) -> Vec<String> {
    let re = Regex::new(r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}").unwrap();

    let dates: Vec<String> = re.find_iter(dt_range.as_str())
        .map(|m| m.as_str().to_string())
        .collect();
    dates
    
}

fn get_datetime_range() -> Vec<String> {
    println!("press 'q' to quit at any time.");
    loop {
        let dt_range = read_string("Enter a date time range (YYYY-MM-DD HH:MM:SS --- YYYY-MM-DD HH:MM:SS):\n");
        if dt_range.trim() == "q" {
            let empty = vec!("".to_string());
            return empty
        }
        let dates = parse_datetime_range(dt_range);
        match dates.len() {
            0 => {
                println!("No datetime range given, please follow the format provided.");
                continue;
            }
            1 => {
                println!("Only 1 datetime given, please give two datetimes to form a range you want to query.");
                continue;
            }
            2 => {
                return dates
            }
            _ => {
                println!("Too many datetimes provided. Please enter two datetimes to form a range you want to query.")
            }
        }
        
    }

}

fn write_all_records(sys: &mut SystemData, conn: Arc<Mutex<Connection>>, p: bool) {

    sys.refresh_all();
            let dt = Local::now();
            let dt = dt.format("%Y-%m-%d %H:%M:%S").to_string();

            let ram_record = RAMRecord {
                datetime: dt.clone(),
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
                total_swap: sys.total_swap(),
                used_swap: sys.used_swap()
            };
            let _ = ram_record.write_to_db(conn.clone());
            if p {
                println!("{}", ram_record);
            }

            let disks = Disks::new_with_refreshed_list();
            for disk in &disks {
                let disk_record = DiskRecord {
                    datetime: dt.clone(),
                    name: disk.name().to_str().unwrap().to_string(),
                    total: disk.total_space(),
                    available: disk.available_space()
                };
                let _ = disk_record.write_to_db(conn.clone());
                if p {
                    println!("{}", disk_record);
                }
            }
            
            let components = Components::new_with_refreshed_list();
            for component in &components {
                let component_record = ComponentRecord {
                    datetime: dt.clone(),
                    label: component.label().to_string(),
                    temp: component.temperature()
                };
                let _ = component_record.write_to_db(conn.clone());
                if p {
                    println!("{}", component_record);
                }
            }

}