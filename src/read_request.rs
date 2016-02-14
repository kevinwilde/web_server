// Christopher Chen and Kevin Wilde
// csc404, kjw731
// EECS 395

#[doc="

"]

pub fn get_file_path_from_request(line: String) -> Option<String> {
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