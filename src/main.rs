

use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(stream: TcpStream) {
    // ...
    println!("Correct request received!");
}

fn main() {
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

	// accept connections and process them, spawning a new thread for each one
	for stream in listener.incoming() {
	    match stream {
	        Ok(stream) => {
	            thread::spawn(move|| {
	                // connection succeeded
	                handle_client(stream)
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
