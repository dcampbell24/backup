//! Different backup strategies.

use std::{
    fs, io, os,
    process::{Command, Stdio},
};

use chrono::{Datelike, Utc};
use nix::unistd::Uid;

fn main() -> Result<(), io::Error> {
    if !Uid::effective().is_root() {
        panic!("You must run this executable with root permissions.");
    }

    backup_btrfs()
}

// sudo mount -a
// # In /etc/fstab
// UUID=4a386a65-8744-46ad-81c2-272bf251fcda /mnt/raid10 btrfs auto,nofail,defaults,lazytime,compress=zstd 0       0

// # The first time.
// btrfs subvolume snapshot -r /home /home/backup
// sudo btrfs send /home/backup | sudo btrfs receive /mnt/raid10
// sudo mv /mnt/raid10/backup /storage/@home-YYYY-mm-dd

// # After the first time.
// btrfs subvolume snapshot -r /home /home/backup-new
// -> btrfs send -p /home/backup /home/backup-new | btrfs receive /mnt/raid10
// sudo mv /mnt/raid10/backup-new /mnt/raid10/@home-YYYY-mm-dd
// sudo btrfs subvolume delete /home/backup
// mv /home/backup-new /home/backup

// sudo umount /mnt/raid10

/// Basic snapshot-style btrfs backup script.
fn backup_btrfs() -> Result<(), io::Error> {
    println!("mount -a");
    Command::new("mount").arg("-a").status()?;

    println!("btrfs subvolume snapshot -r /home /home/backup-new");
    Command::new("btrfs")
        .args(["subvolume", "snapshot", "-r", "/home", "/home/backup-new"])
        .status()?;

    println!("btrfs send -p /home/backup /home/backup-new | btrfs receive /mnt/raid10");
    let send = Command::new("btrfs")
        .args(["send", "-p", "/home/backup", "/home/backup-new"])
        .stdout(Stdio::piped())
        .spawn()?;
    let mut receive = Command::new("btrfs")
        .args(["receive", "/mnt/raid10"])
        .stdin(send.stdout.unwrap())
        .spawn()?;
    receive.wait()?;

    let date = Utc::now();
    let backup = format!(
        "/mnt/raid10/@home-{}-{}-{}",
        date.year(),
        date.month(),
        date.day()
    );
    println!("mv /mnt/raid10/backup-new {backup}");
    fs::rename("/mnt/raid10/backup-new", backup)?;

    println!("btrfs subvolume delete /home/backup");
    Command::new("btrfs")
        .args(["subvolume", "delete", "/home/backup"])
        .status()?;

    println!("mv /home/backup-new /home/backup");
    fs::rename("/home/backup-new", "/home/backup")?;

    println!("umount /mnt/raid10");
    Command::new("umount").arg("/mnt/raid10").status()?;

    Ok(())
}

/// Basic snapshot-style rsync backup script.
fn _backup_rsync() -> Result<(), io::Error> {
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
            &backup,
        ])
        .status()
        .expect("failed to execute rsync");

    fs::remove_file(last)?;
    os::unix::fs::symlink(&backup, last)?;
    Ok(())
}
