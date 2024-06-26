use crate::plugin::Plugin;
use dialoguer::Select;
use clap::{Args, Subcommand};
use eyre::{Context, Result};
use glob::glob;
use regex::Regex;
use std::{f32::consts::E, fs};

pub struct Dotnet;

#[derive(Args, Debug)]
pub struct DotnetCommand {
    #[command(subcommand)]
    command: DotnetSubcommands,
}

fn lauch_settings_regex() -> Result<regex::Regex> {
    Regex::new(r###"^\s{4,6}\"(?<lsn>[^:]*)\": \{$"###)
        .with_context(|| "Something wrong with the launch setting regex")
}

fn env_var_regex() -> Result<regex::Regex> {
    Regex::new(r###"environ.*Varia"###)
        .with_context(|| "Something wrong with the environment variable regex")
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
    let regex = lauch_settings_regex()?;
    let env_regex = env_var_regex()?;

    let filtered: Vec<&str> = g.iter()
        .map(|f|
            f.split("\n").filter(|l| regex.is_match(l) && !env_regex.is_match(l)))
        .flatten()
        .collect();
    let launch_settings: Vec<&str> = filtered
        .iter()
        .map(|l| regex.captures_iter(l).map(|c| c.name("lsn").unwrap().as_str()))
        .flatten()
        .collect();
    let selected = Select::new()
        .with_prompt("Select launch setting")
        .items(&launch_settings)
        .interact()?;

    println!("{}", launch_settings[selected]);

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
    fn test_the_regex_is_match() {
        let regex = lauch_settings_regex().expect("Regex fails");

        assert!(regex.is_match("    \"Erp - Dev\": {"));
        assert!(!regex.is_match("\"Erp - Dev\": {"));
        assert!(!regex.is_match("\"Erp - Dev\": \"something\" ,"));
        assert!(!regex.is_match("\"Erp - Dev\": \"something\" {"));
    }

    #[test]
    fn test_the_regex_capturing() {
        let regex = lauch_settings_regex().expect("Regex fails");
        let capture = regex.captures("    \"Erp - Dev\": {").unwrap();

        assert_eq!(capture["lsn"], "Erp - Dev".to_string());
    }
}
