use std::{io, process::Command};

pub fn fortune(max_length: u64) -> io::Result<String> {
    let result = Command::new("fortune")
        .args(["-n", &max_length.to_string(), "-e"])
        .output()?
        .stdout;

    match String::from_utf8(result) {
        Ok(v) => Ok(v),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
