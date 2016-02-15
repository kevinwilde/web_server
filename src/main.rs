// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="
Still need to handle directories vs files
Spaces in path
"]

use std::net::TcpListener;
use std::thread;

mod response;

fn main() {
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

	// accept connections and process them, spawning a new thread for each one
	for stream in listener.incoming() {
	    match stream {
	        Ok(stream) => {
	            thread::spawn(move|| {
	                // connection succeeded
	                response::handle_client(stream);
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
