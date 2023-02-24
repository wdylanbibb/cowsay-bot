use std::{
    io,
    process::{Command, Stdio},
    str,
};

pub fn cowsay() -> io::Result<String> {
    let cowdir = Command::new("ls")
        .arg("/usr/share/cows")
        .stdout(Stdio::piped())
        .spawn()?;
    let random_cow = Command::new("shuf")
        .arg("-n1")
        .stdin(cowdir.stdout.unwrap())
        .output()?
        .stdout;
    let random_cow = match str::from_utf8(&random_cow) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let fortune = Command::new("fortune").stdout(Stdio::piped()).spawn()?;
    let result = Command::new("cowsay")
        .args(["-f", random_cow.trim()])
        .stdin(fortune.stdout.unwrap())
        .output()?
        .stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                cowsay()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
