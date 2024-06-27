use crate::doctor::{DoctorFailure, DoctorSuccess};
// use std::result::Result::Ok;
use crate::plugin::Plugin;
use clap::{Args, Subcommand};
use dialoguer::Select;
use eyre::{Context, ContextCompat, Ok, Result};
use glob::glob;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

struct ProjectsWithLaunchSettings {
    path_to_csproj: PathBuf,
    launch_settings: Vec<String>,
}

fn get_projects_with_launch_settings() -> Result<Vec<ProjectsWithLaunchSettings>> {
    let projects = glob("**/*.csproj")?;
    let g = projects
        .into_iter()
        .map(|p| {
            let path = p?;
            let settings = get_launch_setting_names(&path);
            Ok((path, settings))
        })
        .filter(|l| l.as_ref().is_ok_and(|m| m.1.is_ok()))
        .map(|l| {
            let k = l.unwrap();
            ProjectsWithLaunchSettings {
                path_to_csproj: k.0,
                launch_settings: k.1.unwrap(),
            }
        })
        .collect();

    Ok(g)
}

fn get_launch_setting_names(project_path: &Path) -> Result<Vec<String>> {
    let mut project_folder = project_path.parent().context("msg")?.to_owned();
    project_folder.push("Properties");
    project_folder.push("launchSettings.json");
    let file = fs::read_to_string(&project_folder)?;

    let regex = lauch_settings_regex()?;
    let env_regex = env_var_regex()?;
    let filtered: Vec<&str> = file
        .split("\n")
        .filter(|l| regex.is_match(l) && !env_regex.is_match(l))
        .collect();
    let names = filtered
        .iter()
        .map(|l| {
            regex
                .captures(l)
                .map(|c| c.name("lsn").unwrap().as_str().to_string())
                .unwrap()
        })
        .collect();
    Ok(names)
}

fn dotnet_run(additional_params: Option<String>) -> Result<()> {
    let launch_settings = get_projects_with_launch_settings()?;
    let project_items: Vec<_> = launch_settings
        .iter()
        .map(|l| l.path_to_csproj.to_str().unwrap())
        .collect();
    let selected_proj = Select::new()
        .with_prompt("Select project")
        .items(&project_items)
        .interact()?;

    let select_launch = &launch_settings[selected_proj].launch_settings;
    let selected_launch_name = Select::new()
        .with_prompt("Select project")
        .items(&select_launch)
        .interact()?;

    let launch_setting_name = &launch_settings[selected_proj].launch_settings[selected_launch_name];
    let project_path = launch_settings[selected_proj]
        .path_to_csproj
        .to_str()
        .context("msg")?;
    let mut args = ["--launch-profile=\"".to_string() + launch_setting_name + "\""].to_vec();
    args.push("--project=\"".to_string() + project_path + "\"");
    if additional_params.is_some() {
        args.push(additional_params.unwrap())
    };

    Command::new("dotnet run")
        .args(args)
        .spawn()
        .expect("Could not run dotnet command");
    Ok(())
}

impl Plugin for Dotnet {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        let mut res = Vec::new();
        res.push(is_dotnet_installed());
        res
    }
}

#[derive(Subcommand, Debug)]
enum DotnetSubcommands {
    Run { additional_params: Option<String> },
}

fn is_dotnet_installed() -> core::result::Result<DoctorSuccess, DoctorFailure> {
    let cmd_result = Command::new("dotnet")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();

    let res = match cmd_result {
        core::result::Result::Ok(_) => core::result::Result::Ok(DoctorSuccess {
            message: format!("dotnet is installed"),
            plugin: "dotnet".to_string(),
        }),
        core::result::Result::Err(_) => core::result::Result::Err(DoctorFailure {
            message: format!("Dotnet is not available. Make sure it is installed"),
            plugin: "dotnet".to_string(),
        }),
    };
    res
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
