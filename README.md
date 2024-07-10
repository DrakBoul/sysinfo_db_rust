# SysInfoDB
## Introduction
SysInfoDB is a small rust program that utilizes crates like rusqlite and sysinfo to track and store system information in an sqlite database. It has a simple command line interface that allows the user to choose when to record data, when to stop recording data, and how to query the database.
## How To Use
Clone the repository to your local machine by navigating to your desired directory and running the following command:
```
git clone git@github.com:DrakBoul/sysinfo_db_rust.git
```
Next change your current working directory to sysinfo_db_rust:
```
cd sysinfo_db_rust
```
Now make a new directory called "data" to hold your sqlite database. <br> 

```
mkdir data
```
<br>

__Note: Make sure the folder is named "data". The program looks specifically in this folder for the database file. If you wish to change the location feel free to modify the file path of the database in the source code.__ <br> <br>

Then run the program by running cargo run command:
```
cargo run 
```
<br> 
No need to create the database file yourself, the program will do it for you automatically within the "data" folder you just created. <br><br>
And your off!!! feel free to play around with the program and monitor your system.