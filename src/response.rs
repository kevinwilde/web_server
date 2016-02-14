// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="

"]

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind, Read, Write};
use std::net::TcpStream;

pub fn handle_client(stream: TcpStream) {
    let mut request_buf = String::new();
    let mut mut_stream = stream;

    if let Ok(n) = mut_stream.read_to_string(&mut request_buf) {
		if n > 0 {
			let response_code: usize;

			match get_file_path_from_request(request_buf.to_string()) {
				Some(path) => {
					
					match File::open(clean_path(path.to_string())) {
						Ok(file) => {
							response_code = 200;
							let file_type = get_file_type(path.to_string());
							let mut file_content_buf = String::new();
							let mut mut_file = file;
							if let Ok(n) = mut_file.read_to_string(&mut file_content_buf) {
								print_ok_response(file_type.to_string(), n, file_content_buf);
							}
						},

						Err(e) => {
							match e.kind() {
								ErrorKind::NotFound =>  {
									response_code = 404;
									println!("404 File Not Found");
								},
								ErrorKind::PermissionDenied => {
									response_code = 403;
									println!("403 Permission Denied");
								},
								_ => panic!("Unknown error in opening file")
							}
						}
				    };
				},

				None =>  {
					response_code = 400;
					println!("400 Bad Request")
				}
		    }
			log_request_and_response("date".to_string(), request_buf, response_code);
		}
    }
}

fn get_file_path_from_request(line: String) -> Option<String> {
    let v = split_request(line);
    if v[0].to_uppercase() == "GET" && v[2].to_uppercase().contains("HTTP") {
        Some(v[1].to_owned())
    } else {
        None
    }
}

fn split_request(line: String) -> Vec<String> {
    line.split(|c: char| c.is_whitespace())
    	.map(|s| s.to_owned())
    	.filter(|s| s.len() > 0)
    	.collect()
}

fn clean_path(path: String) -> String {
	if &path[0..1] == "/" {
		path[1..].to_string()
	} else {
		path
	}
}

// File type is either html for files whose suffix is .html or plain for all others
fn get_file_type(path: String) -> String {
	if path.trim()[path.len()-4..].to_lowercase() == "html" {
		"text/html".to_string()
	} else {
		"text/plain".to_string()
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

fn log_request_and_response(date: String, request: String, response_code: usize) {
	let path_to_log_file = "log.txt";
	let f = match OpenOptions::new().write(true).append(true).open(path_to_log_file) {
		Ok(f) => f,
		Err(e) => panic!("Error opening log file: {}", e)
	};
	let mut writer = BufWriter::new(&f);
	match writer.write_fmt(format_args!("Date: {}, Request: {}, Response Code: {}", date, request, response_code)) {
		Ok(_) => return,
		Err(e) => panic!("Error writing to log file: {}", e)
	}
	// match OpenOptions::new().read(true).write(true).open(path_to_log_file); {
	// 	Ok(f) => f.write_fmt(format_args!("Date: {}, Request: {}, Response Code: {}", date, request, response_code)),
	// 	Err(e) => panic!("Error writing to log file")
	// };
}


#[cfg(test)]
mod response_tests {
	
	use super::clean_path;
	use super::get_file_type;

	#[test]
	fn clean_path_test_remove_front_slash() {
		assert_eq!(clean_path("/src/main.rs".to_string()), 
			"src/main.rs".to_string());
		assert_eq!(clean_path("/src/tmp/some/dir/hello.py".to_string()), 
			"src/tmp/some/dir/hello.py".to_string());
	}

	#[test]
	fn clean_path_test_no_front_slash() {
		assert_eq!(clean_path("src/main.rs".to_string()), 
			"src/main.rs".to_string());
		assert_eq!(clean_path("src/tmp/some/dir/hello.py".to_string()), 
			"src/tmp/some/dir/hello.py".to_string());
	}

	#[test]
	fn get_file_type_test_html() {
		assert_eq!(get_file_type("blah.html".to_string()), 
			"text/html".to_string());
		assert_eq!(get_file_type("test/some/dir/again/index.html".to_string()), 
			"text/html".to_string());
		assert_eq!(get_file_type("a.html".to_string()), 
			"text/html".to_string());
		assert_eq!(get_file_type("alskgjhaksjghlaskdjagl.html".to_string()), 
			"text/html".to_string());
	}

	#[test]
	fn get_file_type_test_plain() {
		assert_eq!(get_file_type("blah.txt".to_string()), 
			"text/plain".to_string());
		assert_eq!(get_file_type("test/some/dir/again/index.txt".to_string()), 
			"text/plain".to_string());
		assert_eq!(get_file_type("other.css".to_string()), 
			"text/plain".to_string());
		assert_eq!(get_file_type("some.js".to_string()), 
			"text/plain".to_string());
		assert_eq!(get_file_type("another.py".to_string()), 
			"text/plain".to_string());
		assert_eq!(get_file_type("what.pdf".to_string()), 
			"text/plain".to_string());
	}

}