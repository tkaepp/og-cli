use std::ffi::OsStr;

use clap::builder::TypedValueParser;
use dialoguer::MultiSelect;
use expanduser::expanduser;
use eyre::Result;
use ssh_key::PublicKey;

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
fn check_ssh_keys() -> Result<()> {
    let ssh_dir = expanduser("~/.ssh")?; // move this check to the doctor

    let public_keys: Vec<PublicKey> = ssh_dir.read_dir()?
        .filter_map(|r| r.ok())
        .map(|r| (r.path().extension().and_then(OsStr::to_str).map(|s| s.to_string()), r))
        .filter(|(extension, _)| extension.as_ref().map(|c| c == "pub").unwrap_or(false))
        .map(|(_, e)| PublicKey::read_openssh_file(e.path().as_path()))
        .filter_map(|p| p.ok())
        .collect();

    let selection = MultiSelect::new()
        .with_prompt("Select ssh keys to use")
        .items(&public_keys)
        .interact()
        .unwrap();
    println!("You chose:");

    for i in selection {
        println!("{}", public_keys[i].to_string());
    }
    Ok(())
}

#[cfg(target_family = "windows")]
fn check_ssh_keys() {}