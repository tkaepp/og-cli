use std::ffi::OsStr;

use clap::builder::TypedValueParser;
use dialoguer::MultiSelect;
use expanduser::expanduser;
use eyre::Result;
use ssh_key::{Algorithm, LineEnding, PrivateKey, PublicKey};
use ssh_key::rand_core::OsRng;

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

fn setup() -> Result<()> {
    check_ssh_keys()
}

#[cfg(target_family = "unix")]
fn check_ssh_keys() -> Result<()> {
    let ssh_dir = expanduser("~/.ssh2")?; // move this check to the doctor

    let public_keys: Vec<PublicKey> = ssh_dir.read_dir()?
        .filter_map(|r| r.ok())
        .map(|r| (r.path().extension().and_then(OsStr::to_str).map(|s| s.to_string()), r))
        .filter(|(extension, _)| extension.as_ref().map(|c| c == "pub").unwrap_or(false))
        .map(|(_, e)| PublicKey::read_openssh_file(e.path().as_path()))
        .filter_map(|p| p.ok())
        .collect();

    if public_keys.is_empty() {
        let creation_options = &["Azure Devops", "Github"];
        let defaults = &[true, true];
        let selection = MultiSelect::new()
            .with_prompt("Should OG create ssh keys for you?")
            .items(&creation_options[..])
            .defaults(&defaults[..])
            .interact()
            .unwrap();

        if selection.is_empty() {
            println!("No git platforms were selected. Abort key creation");
            return Ok(());
        }

        let private_key_ed = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;

        private_key_ed.write_openssh_file(ssh_dir.join("og-ssh").as_path(), LineEnding::LF).expect("key could not be created");
        private_key_ed.public_key().write_openssh_file(ssh_dir.join("og-ssh.pub").as_path()).expect("key could not be created");
    } else {
        let selection = MultiSelect::new()
            .with_prompt("Select ssh keys to use")
            .items(&public_keys)
            .interact()
            .unwrap();
        println!("You chose:");

        for i in selection {
            println!("{}", public_keys[i].to_string());
        }
    }


    Ok(())
}

#[cfg(target_family = "windows")]
fn check_ssh_keys() {}