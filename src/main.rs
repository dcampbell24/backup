//! Different backup strategies.

use std::{fs, io, process::Command, os};

use chrono::{Utc, Datelike};

fn main() -> Result<(), io::Error> {
    backup_btrfs()
}

// sudo mount -a
// # In /etc/fstab
// UUID=4a386a65-8744-46ad-81c2-272bf251fcda /mnt/raid10 btrfs auto,nofail,defaults,lazytime,compress=zstd 0       0

// btrfs subvolume snapshot -r /home /home/backup //
// sudo btrfs send /home/backup | sudo btrfs receive /mnt/raid10 //
// sudo mv /mnt/raid10/backup /storage/@home-2024-07-02 # line added //

// btrfs subvolume snapshot -r /home /home/backup-new
// sudo btrfs send -p /home/backup /home/backup-new | sudo btrfs receive /mnt/raid10
// sudo mv /mnt/raid10/backup-new /mnt/raid10/@home-2024-07-03

// sudo btrfs subvolume delete /home/backup
// mv /home/backup-new /home/backup

// sudo umount /mnt/raid10

/// Basic snapshot-style btrfs backup script
fn backup_btrfs() -> Result<(), io::Error> {





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


/// Basic snapshot-style rsync backup script
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
            &backup
        ])
        .status()
        .expect("failed to execute rsync");

    fs::remove_file(last)?;
    os::unix::fs::symlink(&backup, last)?;
    Ok(())
}
