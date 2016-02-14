// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="

"]

use std::fs::File;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;

use read_request::get_file_path_from_request;

pub fn handle_client(stream: TcpStream) {
    println!("Correct request received!");
    let mut buf = String::new();
    let mut mut_stream = stream;
    if let Ok(n) = mut_stream.read_to_string(&mut buf) {
		if n > 0 {
			match get_file_path_from_request(buf) {
				Some(path) => {
					match File::open(clean_path(path)) {
						Ok(file) => {
							let file_type = "todo";
							let mut file_content_buf = String::new();
							let mut mut_file = file;
							if let Ok(n) = mut_file.read_to_string(&mut file_content_buf) {
								print_ok_response(file_type.to_owned(), n, file_content_buf);
							}
						},
						Err(e) => {
							match e.kind() {
								ErrorKind::NotFound => println!("404 File Not Found"),
								ErrorKind::PermissionDenied => println!("403 Permission Denied"),
								_ => panic!("Unknown error in opening file")
							}
						}
				    };
				}
				None => println!("400 Bad Request")
		    }
		}
    }
}

fn clean_path(path: String) -> String {
	if &path[0..1] == "/" {
		path[1..].to_owned()
	} else {
		path
	}
}

fn print_ok_response(content_type: String, content_length: usize, file_content: String) {
	println!("HTTP/1.0 200 OK");
	println!("csc404-kjw731-web-server/0.1");
	println!("Content-type {}", content_type);
	println!("Content-length: {}", content_length);
	println!("");
	println!("{}", file_content);
}