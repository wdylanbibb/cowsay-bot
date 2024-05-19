use std::{
    io,
    process::{Command, Stdio},
    str,
};

pub fn cowsay(msg: &String, file: Option<&String>) -> io::Result<String> {
    let mut binding = Command::new("cowsay");
    let mut result = &mut binding;
    if let Some(cow) = file {
        result = result.args(["-f", cow, "--"]);
    }
    let result = result.arg(msg).output()?.stdout;
    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                cowsay(msg, file)
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

pub fn random_cowsay_fortune() -> io::Result<String> {
    let cowdir = Command::new("ls")
        .arg("/usr/share/cowsay/cows")
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

    let fortune = Command::new("fortune")
        .args(["-e"])
        .stdout(Stdio::piped())
        .spawn()?;
    let result = Command::new("cowsay")
        .args(["-f", random_cow.trim()])
        .stdin(fortune.stdout.unwrap())
        .output()?
        .stdout;

    match String::from_utf8(result) {
        Ok(v) => {
            if v.len() > 2000 {
                random_cowsay_fortune()
            } else {
                Ok(v)
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
