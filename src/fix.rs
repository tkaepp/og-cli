use clap::Args;
use eyre::{ContextCompat, Result};
use homedir::get_my_home;
use std::{env, fs, process::Command};

use crate::{
    doctor::{DoctorFailure, DoctorSuccess},
    plugin::Plugin,
};

/// Recover the DG CLI (currently macOS only)
#[derive(Args, Debug)]
pub struct FixCommand;

pub struct FixPlugin;

impl FixPlugin {
    pub fn run() -> Result<()> {
        println!("Reinstalling Dg Cli");
        let os = env::consts::OS;
        match os {
            "macos" => {
                let rc_dir = get_my_home()?
                    .context("Could not get home directory")?
                    .join(".dgrc");
                let _ = fs::remove_file(rc_dir);
                let cli_dir = get_my_home()?
                    .context("Could not get home directory")?
                    .join(".dg-cli");
                let _ = fs::remove_dir_all(cli_dir);
                let localdg_dir = get_my_home()?
                    .context("Could not get home directory")?
                    .join(".local/bin/dg");
                let _ = fs::remove_dir_all(localdg_dir);
                let pipx_dir = get_my_home()?
                    .context("Could not get home directory")?
                    .join(".local/pipx");
                let _ = fs::remove_dir_all(pipx_dir);
                println!("attempting to reinstall pipx");
                let uninstallstatus = Command::new("brew")
                    .arg("uninstall")
                    .arg("pipx")
                    .status()
                    .expect("brew command failed to start");

                println!("Uninstall finished with: {uninstallstatus}");

                let message = match fs::remove_dir_all("~/.local/pipx") {
                    Ok(()) => ".dg-cli deleted",
                    Err(_e) => "unknown error",
                };
                println!("{message}");

                let installstatus = Command::new("brew")
                    .arg("install")
                    .arg("pipx")
                    .status()
                    .expect("brew command failed to start");

                println!("Install finished with: {installstatus}");

                Command::new("curl")
                    .arg("-sL")
                    .arg("https://dgcli.platform.prod.int.devinite.com/install.py")
                    .arg("-o")
                    .arg("install.py")
                    .status()
                    .expect("curl command failed to start");

                let clistatus = Command::new("python3")
                    .arg("install.py")
                    .status()
                    .expect("python3 command failed to start");

                fs::remove_file("install.py")?;
                println!("cli install finished with: {clistatus}");
            }
            "windows" => {
                println!("Hi I'm on windows");
            }
            "linux" => {
                println!("Hi I'm on linux");
            }
            _ => {
                println!("InvalidOS, cancelling");
            }
        }
        Ok(())
    }
}

impl Plugin for FixPlugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        Vec::new()
    }
}
