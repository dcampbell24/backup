//! Different backup strategies.

use std::{
    fs, os,
    path::PathBuf,
    process::{Command, Stdio},
};

use chrono::{Datelike, Utc};
use nix::unistd::Uid;

// Run with: sudo --preserve-env env "PATH=$PATH"
fn main() -> anyhow::Result<()> {
    clean_projects()?;

    if Uid::effective().is_root() {
        backup_btrfs()
    } else {
        Err(anyhow::Error::msg(
            "You must run this executable with root permissions.",
        ))
    }
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
// btrfs send -p /home/backup /home/backup-new | btrfs receive /mnt/raid10
// sudo mv /mnt/raid10/backup-new /mnt/raid10/@home-YYYY-mm-dd
// sudo btrfs subvolume delete /home/backup
// mv /home/backup-new /home/backup

// sudo umount /mnt/raid10

/// Basic snapshot-style btrfs backup script.
fn backup_btrfs() -> anyhow::Result<()> {
    println!("mount --all --verbose");
    Command::new("mount")
        .arg("--all")
        .arg("--verbose")
        .status()?;

    let mut sum = 0;
    for file in fs::read_dir("/mnt/raid10")? {
        file?;
        sum += 1;
    }
    if sum == 0 {
        return Err(anyhow::Error::msg("raid10 isn't mounted!"));
    }

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
        "/mnt/raid10/@home-{}-{:02}-{:02}",
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

    println!("umount /mnt/raid10 --verbose");
    Command::new("umount")
        .arg("/mnt/raid10")
        .arg("--verbose")
        .status()?;

    Ok(())
}

fn clean_projects() -> anyhow::Result<()> {
    let mut projects_path = dirs::home_dir().unwrap();
    projects_path = projects_path.join("projects");
    if fs::exists(&projects_path)? {
        for entry in fs::read_dir(&projects_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if fs::exists(path.join("Cargo.toml"))? {
                    cargo_clean(&path)?;
                }
                if fs::exists(path.join("book.toml"))? {
                    mdbook_clean(&path)?;
                }
            }
        }
    }
    Ok(())
}

fn cargo_clean(path: &PathBuf) -> anyhow::Result<()> {
    println!("{path:?} cargo clean");
    Command::new("cargo")
        .current_dir(path)
        .arg("clean")
        .status()?;
    Ok(())
}

fn mdbook_clean(path: &PathBuf) -> anyhow::Result<()> {
    println!("{path:?} mdbook clean");
    Command::new("mdbook")
        .current_dir(path)
        .arg("clean")
        .status()?;
    Ok(())
}
