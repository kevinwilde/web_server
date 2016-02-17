// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="
Assumptions:

The method token in an HTTP request is case sensitive, in accordance with 
https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html#sec5.1.1 
So in the request, the word GET must be in all caps in order for it to be considered a 
valid request.

A path to a directory should end with a /. If it does not, the server will still be 
able to find index.html, index.shtml, or index.txt, but in the case of an html file with 
relative links to external resources, those relative links will be broken. This is in 
accordance with 
http://serverfault.com/questions/587002/apache2-301-redirect-when-missing-at-the-end-of-directory-in-the-url
(this server does not implement the response code 301). When the path to a directory 
ends with a / as it should, the server is of course able to resolve relative links. 
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
                let file = file.clone();
                thread::spawn(move|| {
                    // connection succeeded
                    response::handle_client(stream, &file);
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
