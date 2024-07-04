//! Different backup strategies.

use std::{fs, io, process::Command, os};

use chrono::{Utc, Datelike};

fn main() -> Result<(), io::Error> {
    backup_rsync()
}


/// Basic snapshot-style rsync backup script
fn backup_rsync() -> Result<(), io::Error> {
    let last = "/media/backup/ubuntu/last";

    let mut backup = "/media/backup/ubuntu/".to_string();
    let date = Utc::now();
    backup = format!("{backup}{}-{}-{}", date.year(), date.month(), date.day());

    Command::new("rsync")
        .args([
            "--archive",
            "--partial",
            "--progress",
            "--human-readable",
            "--link-dest=/media/backup/ubuntu/last/", 
            "/home/ubuntu/",
            &backup
        ])
        .status()
        .expect("failed to execute rsync");

    fs::remove_file(last)?;
    os::unix::fs::symlink(&backup, last)?;
    Ok(())
}
