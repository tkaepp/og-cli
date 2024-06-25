use crate::plugin::Plugin;
use clap::{Args, Subcommand};

pub struct DotnetRun;

#[derive(Args, Debug)]
pub struct DotnetRunCommand {
    #[command(subcommand)]
    command: DotnetRunSubcommands,
}

impl DotnetRun {
    pub fn run(cli: DotnetRunCommand) {
        match cli.command {
            DotnetRunSubcommands::Hello { name } => println!("Hi {name}"),
        }
    }
}

impl Plugin for DotnetRun {
    fn doctor(&self) {
        println!("Running the fix doctor");
    }
}

#[derive(Subcommand, Debug)]
enum DotnetRunSubcommands {
    Hello { name: String },
}
