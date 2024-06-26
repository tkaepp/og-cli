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
fn check_ssh_keys() {}

#[cfg(target_family = "windows")]
fn check_ssh_keys() {}