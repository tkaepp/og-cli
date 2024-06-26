use std::ffi::OsStr;
use std::fs::read_dir;

use expanduser::expanduser;

use crate::git;
use crate::git::commands::GitSubCommands;

pub struct Git;


impl Git {
    pub fn run(cli: git::commands::GitCommand) {
        match cli.command {
            GitSubCommands::Setup => setup().unwrap()
        }
    }
}

fn setup() -> Result<(), String> {
    check_ssh_keys();
    Ok(())
}

#[cfg(target_family = "unix")]
fn check_ssh_keys() -> Result<(), String> {
    let ssh_dir = expanduser("~/.ssh").unwrap(); // move this check to the doctor
    match read_dir(ssh_dir) {
        Ok(ssh_dir) => {
            for entry in ssh_dir {
                match entry {
                    Ok(e) => {
                        match e.path().extension().and_then(OsStr::to_str) {
                            Some("pub") => {
                                println!("{}", e.path().to_str().unwrap());
                            }
                            _ => {}
                        }
                    }
                    Err(_) => { println!("Error reading a directory entry") }
                }
            }
        }
        Err(e) => { println!("Error occured {}", e.to_string()) }
    }
    Ok(())
    // ssh_key::public::
    // let existing_keys = PrivateKey::
    //
    // let _ = Select::new()
    //     .with_prompt("Select cluster to sync")
    //     .items(&clusters)
    //     .interact()
    //     .unwrap();
}

#[cfg(target_family = "windows")]
fn check_ssh_keys() {}