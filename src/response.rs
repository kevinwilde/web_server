// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="

"]

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::TcpStream;

extern crate time;

pub fn handle_client(stream: TcpStream) {
    println!("New request");
    let mut stream_reader = BufReader::new(stream);
    let mut request_buf = Vec::new();
    if let Ok(n) = stream_reader.read_until(b'\n', &mut request_buf) {
        if n > 0 {
            let stream = stream_reader.into_inner();
            
            match get_file_path_from_request(String::from_utf8(request_buf.to_owned()).unwrap()) {
                Some(path) => {
                    println!("Got path: {}", path);
                    if is_file(&path) {
                        try_to_open_file(&path, &stream, &request_buf, false);
                    } else {
                        // Path is to a directory
                        // Search for index.html, index.txt, index.shtml etc
                        println!("DIRECTORY!");
                        let mut try = try_to_open_file(&(path.to_string() + "/index.html"), &stream, &request_buf, true);
                        if !try { try = try_to_open_file(&(path.to_string() + "/index.shtml"), &stream, &request_buf, true); }
                        if !try { try_to_open_file(&(path.to_string() + "/index.txt"), &stream, &request_buf, false); }
                    }
                },

                None =>  {
                    let response_code = 400;
                    deliver_error_response(&stream, response_code, "Bad Request".to_string());
                    println!("400 Bad Request");
                    log_request_and_response(time::now().asctime().to_string(), 
                        String::from_utf8(request_buf.to_owned()).unwrap(), 
                        response_code);
                }
            }
        }
    }
    println!("Done with request");
}

fn get_file_path_from_request(line: String) -> Option<String> {
    let v = split_request(line);
    if v.len() >= 3 && v[0].to_uppercase() == "GET" && v[v.len() - 1].to_uppercase().contains("HTTP") {
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
    //let path = path.replace("%20", " ");
    let n = path.len();
    let leading_slash: bool;
    let ending_slash: bool;
    
    if &path[0..1] == "/" {
        leading_slash = true;
    } else {
        leading_slash = false;
    }
    
    if &path[n-1..n] == "/" {
        ending_slash = true;
    } else {
        ending_slash = false;
    }

    if leading_slash {
        if ending_slash {
            path[1..n-1].to_string()
        } else {
            path[1..].to_string()
        }
    } else {
        if ending_slash {
            path[0..n-1].to_string()
        } else {
            path
        }
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

fn try_to_open_file(path: &str, stream: &TcpStream, request_buf: &Vec<u8>, more_files_to_try: bool) -> bool {
    let mut response_code = 0;
    let success: bool;
    match File::open(&path) {
        Ok(file) => {
            success = true;
            println!("200");
            response_code = 200;
            let file_type = get_file_type(path.to_string());
            deliver_ok_response(stream, file_type.to_string(), file);
        },

        Err(e) => {
            success = false;

            if !more_files_to_try {
                match e.kind() {
                    ErrorKind::NotFound =>  {
                        response_code = 404;
                        deliver_error_response(stream, 
                            response_code, 
                            "File Not Found".to_string());
                        println!("404 File Not Found");
                    },
                    ErrorKind::PermissionDenied => {
                        response_code = 403;
                        deliver_error_response(stream, 
                            response_code, 
                            "Permission Denied".to_string());
                        println!("403 Permission Denied");
                    },
                    _ => panic!("Unknown error in opening file")
                }
            }
        }
    }
    if !more_files_to_try {
        log_request_and_response(time::now().asctime().to_string(), 
            String::from_utf8(request_buf.to_owned()).unwrap(), 
            response_code);
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
    // println!("HTTP/1.0 200 OK");
    // println!("csc404-kjw731-web-server/0.1");
    // println!("Content-type: {}", content_type);
    // println!("Content-length: {}", content_length);
    // println!("");
    // println!("{}", file_content);
    
    let size = file.metadata().unwrap().len();
    let header = "HTTP/1.0 200 OK\ncsc404-kjw731-web-server/0.1\nContent-type: ".to_string() 
                    + &content_type + &"\nContent-length: " + &size.to_string() + &"\n\n";

    match stream.write_all(&header.into_bytes()[0..]){
        Ok(_) => {},
        Err(_) => panic!("Error delivering response")
    }

    let mut buf = vec![0;2048];
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
    match stream.write_fmt(format_args!("HTTP/1.0 {} \ncsc404-kjw731-web-server/0.1\n{}\n\n", 
        error_code, error_message)) {
        Ok(_) => {},
        Err(_) => panic!("Error delivering response")
    }
    match stream.write_all(&error_message.into_bytes()[..]) {
        Ok(_) => {},
        Err(_) => panic!("Error delivering response")
   }
}

fn log_request_and_response(date: String, request: String, response_code: usize) {
    let path_to_log_file = "log.txt";
    let f = match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(path_to_log_file) {
        Ok(f) => f,
        Err(e) => panic!("Error opening log file: {}", e)
    };
    let mut writer = BufWriter::new(&f);
    match writer.write_fmt(format_args!("Date: {}\nRequest: {}\nResponse Code: {}\n", 
        date, request, response_code)) {
        Ok(_) => {},
        Err(e) => panic!("Error writing to log file: {}", e)
    }
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