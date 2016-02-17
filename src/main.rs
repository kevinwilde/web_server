// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="
Spaces in path? Split whitespace, check first and last for GET and HTTP, concat the remaining
Content-length: Use chars.count iterator on the file handler. Don't worry about traversing twice.
Log file -- multiple threads trying to write: use mutex, test with random delays and multiple threads
"]

use std::fs::{File, OpenOptions};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

mod response;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let path_to_log_file = "log.txt";
    let file: Arc<Mutex<File>>;
    match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(path_to_log_file) {
        Ok(f) => file = Arc::new(Mutex::new(f)),
        Err(e) => panic!("Error opening log file: {}", e)
    };

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let file_clone = file.clone();
                thread::spawn(move|| {
                    // connection succeeded
                    response::handle_client(stream, &file_clone);
                });
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    // close the socket server
    drop(listener);
}
