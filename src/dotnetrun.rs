use crate::plugin::Plugin;
use clap::{Args, Subcommand};
use glob::glob;
use regex::Regex;
use std::fs;

pub struct Dotnet;

#[derive(Args, Debug)]
pub struct DotnetCommand {
    #[command(subcommand)]
    command: DotnetSubcommands,
}

impl Dotnet {
    pub fn run(cli: DotnetCommand) -> Result<(), String> {
        match cli.command {
            DotnetSubcommands::Run { additional_params } => dotnet_run(&additional_params)
        }
    }
}

fn dotnet_run(additional_params: &str) -> Result<(), String> {
        let regex = Regex::new(r###"\s*\"[a-zA-Z()]*\": \\{$"###).unwrap();
        let g = glob("**/Properties/launchSettings.json")?
            .into_iter()
            .map(|f| fs::read_to_string(f.unwrap()).expect("Could not read file"));

        Ok(())

}

impl Plugin for Dotnet {
    fn doctor(&self) {
        println!("Running the fix doctor");
    }
}

#[derive(Subcommand, Debug)]
enum DotnetSubcommands {
    Run { additional_params: String },
}
