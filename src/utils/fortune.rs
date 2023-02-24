pub fn fortune() -> io::Result<String> {
    let result = Command::new("fortune").output()?.stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                fortune()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
