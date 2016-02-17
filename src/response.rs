// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

use std::fs::{File};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

extern crate time;

pub fn handle_client(stream: TcpStream, log: &Arc<Mutex<File>>) {
    println!("New request");

    let mut stream_reader = BufReader::new(stream);
    let mut request_buf = String::new();

    if let Ok(_) = stream_reader.read_line(&mut request_buf) {
        
        let stream = stream_reader.into_inner();
        print!("{}", &request_buf);
        
        match get_file_path_from_request(request_buf.to_string()) {

            Some(path) => {

                if is_file(&path) {
                    let _ = try_to_open_file(&path, &stream, &request_buf, log, false);
                } else {
                    // Path is to a directory

                    // Edge Case: Look for index.{html,shtml,txt} in current directory
                    // So add / to end of directory unless looking in current directory
                    let mut path = path.to_string();
                    if path.len() > 0 {
                        path = path + "/";
                    }

                    let mut try = try_to_open_file(&(path.to_string() + "index.html"), 
                        &stream, &request_buf, log, true);
                    if !try { try = try_to_open_file(&(path.to_string() + "index.shtml"), 
                        &stream, &request_buf, log, true); }
                    if !try { try_to_open_file(&(path.to_string() + "index.txt"), &stream, 
                        &request_buf, log, false); }
                }
            },

            None =>  {
                let response_code = 400;
                deliver_error_response(&stream, response_code, "Bad Request".to_string());
                println!("400 Bad Request");
                log_request_and_response(log, time::now().asctime().to_string(), 
                    request_buf.to_string(), 
                    response_code);
            }
        }
    }
    println!("Done with request\n");
}

fn get_file_path_from_request(line: String) -> Option<String> {
    let v = split_request(line);
    //HTTP methods are case sensitive. So, GET must be all-caps.
    if v.len() >= 3 && v[0] == "GET" && &v[1][0..1] == "/" 
      && v[v.len() - 1].to_uppercase().contains("HTTP") {
        Some(clean_path(v[1..v.len() - 1].join(" ").to_string()))
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
    let path = path.replace("%20", " ");
    let n = path.len();

    if path == "/" {
        return "".to_string();
    }
    
    // Remove leading "/" (always present)
    // and remove ending "/" (if present)
    if &path[n-1..n] == "/" {
        return path[1..n-1].to_string();
    } else {
        return path[1..].to_string();
    }
}

fn is_file(path: &str) -> bool {
    for ch in path.chars().rev() {
        if ch == '.' {
            return true;
        } else if ch == '/' {
            return false;
        }
    }
    return false;
}

fn try_to_open_file(path: &str, stream: &TcpStream, request_buf: &String, 
    log: &Arc<Mutex<File>>, more_files_to_try: bool) -> bool {
    
    let response_code: usize;
    let success: bool;
    
    match File::open(&path) {
        Ok(file) => {
            success = true;
            println!("200");
            response_code = 200;
            let file_type = get_file_type(path.to_string());
            deliver_ok_response(stream, file_type.to_string(), file);
            log_request_and_response(log, time::now().asctime().to_string(), 
                request_buf.to_string(), 
                response_code);
        },

        Err(e) => {
            success = false;

            if !more_files_to_try {
                match e.kind() {
                    ErrorKind::NotFound =>  {
                        response_code = 404;
                        deliver_error_response(stream, 
                            response_code, 
                            "Not Found".to_string());
                        println!("404 Not Found");
                    },

                    ErrorKind::PermissionDenied => {
                        response_code = 403;
                        deliver_error_response(stream, 
                            response_code, 
                            "Forbidden".to_string());
                        println!("403 Forbidden");
                    },

                    _ => panic!("Unknown error in opening file")
                }
                log_request_and_response(log, time::now().asctime().to_string(), 
                    request_buf.to_string(), 
                    response_code);
            }
        }
    }
    success
}

// File type is either:
//    text/html for files whose suffix is .html 
//    text/javascript for files whose suffix is .js
//    text/css for files whose suffix is .css
//    text/plain for all others
fn get_file_type(path: String) -> String {
    if path.trim()[path.len()-5..].to_lowercase() == ".html" {
        "text/html".to_string()
    } else if path.trim()[path.len()-3..].to_lowercase() == ".js" {
        "text/javascript".to_string()
    } else if path.trim()[path.len()-4..].to_lowercase() == ".css" {
        "text/css".to_string()
    } else {
        "text/plain".to_string()
    }
}

fn deliver_ok_response(mut stream: &TcpStream, content_type: String,  mut file: File) {
    let size = file.metadata().unwrap().len();
    let header = "HTTP/1.0 200 OK\ncsc404-kjw731-web-server/0.1\nContent-type: ".to_string() 
                    + &content_type + &"\nContent-length: " + &size.to_string() + &"\n\n";

    stream.write_all(&header.into_bytes()[0..]).expect("Error delivering response");

    let mut buf = vec![0; 1024];
    while let Ok(n) = file.read(&mut buf) {
        if n <= 0 {
            break;
        }
        match stream.write_all(&buf[0..n]) {
            Ok(_) => {},
            Err(_) => break
       }
    }
}

fn deliver_error_response(mut stream: &TcpStream, error_code: usize, error_message: String) {
    stream.write_fmt(format_args!("HTTP/1.0 {} {} \ncsc404-kjw731-web-server/0.1\n\n", 
        error_code, error_message)).expect("Error delivering response");
    stream.write_fmt(format_args!("{}\n", error_message)).expect("Error delivering response");
}

fn log_request_and_response(log: &Arc<Mutex<File>>, date: String, request: String, response_code: usize) {
    let mut log_guard = log.lock().unwrap();
    log_guard.write_fmt(format_args!("Date: {}\r\nRequest: {}Response Code: {}\r\n\r\n", 
        date, request, response_code)).expect("Error writing to log file");
}


#[cfg(test)]
mod response_tests {
    
    use super::clean_path;
    use super::get_file_path_from_request;
    use super::is_file;
    use super::get_file_type;

    #[test]
    fn clean_path_test_remove_front_slash() {
        assert_eq!(clean_path("/src/main.rs".to_string()), 
            "src/main.rs".to_string());
        assert_eq!(clean_path("/src/tmp/some/dir/hello.py".to_string()), 
            "src/tmp/some/dir/hello.py".to_string());
    }

    #[test]
    fn clean_path_test_remove_end_slash() {
        assert_eq!(clean_path("/src/".to_string()), 
            "src".to_string());
        assert_eq!(clean_path("/src/tmp/some/dir/".to_string()), 
            "src/tmp/some/dir".to_string());
    }

    #[test]
    fn clean_path_test_spaces() {
        assert_eq!(clean_path("/this%20has%20spaces.txt".to_string()),
            "this has spaces.txt".to_string());
    }

    #[test]
    fn get_file_path_test_none() {
        assert_eq!(None, get_file_path_from_request("GET /test.txt".to_string()));
        assert_eq!(None, get_file_path_from_request("PUT /test.txt HTTP".to_string()));
        assert_eq!(None, get_file_path_from_request("get /test.txt HTTP".to_string()));
        assert_eq!(None, get_file_path_from_request("GET test.txt HTTP".to_string()));
    }

    #[test]
    fn get_file_path_test_some() {
        assert_eq!(Some("test.txt".to_string()), 
            get_file_path_from_request("GET /test.txt HTTP".to_string()));
        assert_eq!(Some("test.txt".to_string()), 
            get_file_path_from_request("GET /test.txt/ http".to_string()));
        assert_eq!(Some("this has spaces.css".to_string()), 
            get_file_path_from_request("GET /this has spaces.css HtTp".to_string()));
        assert_eq!(Some("".to_string()), 
            get_file_path_from_request("GET / HTTP".to_string()));
    }

    #[test]
    fn is_file_test_true() {
        assert!(is_file("test.txt"));
        assert!(is_file("index.html"));
        assert!(is_file("some/dir/test.txt"));
        assert!(is_file("client/scipts/app.js"));
    }

    #[test]
    fn is_file_test_false() {
        assert_eq!(false, is_file(""));
        assert_eq!(false, is_file("dir/"));
        assert_eq!(false, is_file("some/dir/"));
        assert_eq!(false, is_file("dir"));
        assert_eq!(false, is_file("some/dir"));
    }

    #[test]
    fn get_file_type_test_web() {
        // HTML
        assert_eq!("text/html".to_string(), 
            get_file_type("blah.html".to_string()));
        assert_eq!("text/html".to_string(), 
            get_file_type("test/some/dir/again/index.html".to_string()));
        assert_eq!("text/html".to_string(), 
            get_file_type("a.html".to_string()));
        assert_eq!("text/html".to_string(), 
            get_file_type("alskgjhaksjghlaskdjagl.html".to_string()));

        // css
        assert_eq!("text/css".to_string(), 
            get_file_type("other.css".to_string()));

        // Javascript
        assert_eq!("text/javascript".to_string(), 
            get_file_type("some.js".to_string()));
    }

    #[test]
    fn get_file_type_test_plain() {
        assert_eq!("text/plain".to_string(), 
            get_file_type("blah.txt".to_string()));
        assert_eq!("text/plain".to_string(), 
            get_file_type("test/some/dir/again/index.txt".to_string()));
        assert_eq!("text/plain".to_string(), 
            get_file_type("another.py".to_string()));
        assert_eq!("text/plain".to_string(), 
            get_file_type("what.pdf".to_string()));
    } 
}