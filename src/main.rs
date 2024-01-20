//! Store pkgs for backup.
use std::error::Error;
use std::fs::{File, self};
use std::io::Write as _;
use std::{process::Command, fmt::Write};

use chrono::{Utc, Datelike};
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    let output = Command::new("apt")
        .args([
            "list",
            "--manual-installed",
        ])
        .output()
        .expect("failed to execute apt");

    assert!(output.status.success());

    let re_1 = Regex::new(r"^(?<pkg>.*)/.*$")?;
    let re_2 = Regex::new(r"^.*\[.*local.*\]$")?;
    // io::stderr().write_all(&output.stderr)?;
    let mut pkgs = "sudo apt install ".to_string();
    let output = String::from_utf8(output.stdout)?;
    for line in output.split("\n") {
        if let Some(cap) = re_1.captures(line) {
            if re_2.is_match(line) {
                println!("{line}");
            } else {
                let pkg = cap.name("pkg").unwrap().as_str();
                write!(&mut pkgs, "{pkg} ")?;
            }
        }
    }
    writeln!(&mut pkgs)?;

    let date = Utc::now();
    let backup = format!("./backup-pkgs-{}-{}-{}.txt", date.year(), date.month(), date.day());
    let mut file = File::create(&backup)?;
    file.write_all(pkgs.as_bytes())?;

    for entry in fs::read_dir("/opt")? {
        println!("{:?}", entry?.file_name());
    }

    Ok(())
}
