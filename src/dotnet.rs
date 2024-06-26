use crate::plugin::Plugin;
use clap::{Args, Subcommand};
use eyre::Result;
use glob::glob;
use regex::Regex;
use std::fs;

pub struct Dotnet;

#[derive(Args, Debug)]
pub struct DotnetCommand {
    #[command(subcommand)]
    command: DotnetSubcommands,
}

fn lauch_settings_regex() -> Result<regex::Regex> {
    Regex::new(r###"\"\s{4,}"###)?
}

pub struct LauchSettingsRegex {
    expr: Regex,
}

impl Dotnet {
    pub fn run(cli: DotnetCommand) -> Result<()> {
        match cli.command {
            DotnetSubcommands::Run { additional_params } => dotnet_run(additional_params),
        }
    }
}

fn dotnet_run(additional_params: Option<String>) -> Result<()> {
    let g: Vec<_> = glob("**/Properties/launchSettings.json")?
        .into_iter()
        .map(|f| fs::read_to_string(f.unwrap()).expect("Could not read file"))
        .collect();
    // let regex = Regex::new(r###"\s*\"[a-zA-Z()]*\": \{$"###)?;
    let regex = Regex::new(r###"\"\s{4,}"###)?;

    let filtered: Vec<_> = g.iter().filter(|l| regex.is_match(l)).collect();
    filtered.iter().for_each(|f| println!("{}", f));

    Ok(())
}

impl Plugin for Dotnet {
    fn doctor(&self) {
        println!("Running the fix doctor");
    }
}

#[derive(Subcommand, Debug)]
enum DotnetSubcommands {
    Run { additional_params: Option<String> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_regex() {}
}
