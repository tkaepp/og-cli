use crate::plugin::Plugin;
use clap::{Args, Subcommand};
use crate::doctor::{DoctorFailure, DoctorSuccess};
use std::env;
use std::fs;
use std::process::Command;

pub struct Fix;

#[derive(Args, Debug)]
pub struct FixCommand {
    #[command(subcommand)]
    command: FixSubcommands,
}

impl Fix {
    pub fn run(cli: FixCommand) {
        match cli.command {
            FixSubcommands::Hello { name } => println!("Hi {name}"),
            FixSubcommands::Reinstall => 
            { 
                println!("Reinstalling Dg Cli");
                let os = env::consts::OS;
                match os{
                    "macos"=>{
                        Command::new("rm")
                            .arg("-rf ~/.dgrc")
                            .status()
                            .expect("rm does not work");
                        Command::new("rm")
                            .arg("-rf ~/.dg-cli")
                            .status()
                            .expect("rm does not work");
                        Command::new("rm")
                            .arg("-rf ~/.local/bin/dg")
                            .status()
                            .expect("rm does not work");
                        println!("attempting to reinstall pipx");
                        let uninstallstatus = Command::new("brew")
                            .arg("uninstall")
                            .arg("pipx")
                            .status()
                            .expect("brew command failed to start");

                            println!("Uninstall finished with: {uninstallstatus}");

                            let message = match fs::remove_dir_all("~/.local/pipx"){
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

                            println!("cli install finished with: {clistatus}");

                    },
                    "windows"=>{
                        println!("Hi I'm on windows");
                    },
                    "linux"=>{
                        println!("Hi I'm on linux");
                    },
                    _ =>{
                        println!("InvalidOS, cancelling");
                    },
                }
            },
        }
    }
}

impl Plugin for Fix {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        Vec::new()
    }
}

#[derive(Subcommand, Debug)]
enum FixSubcommands {
    Hello { name: String },
    Reinstall,
}
