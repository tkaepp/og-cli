use std::process::Command;

use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::git::Git;
use crate::plugin::Plugin;

impl Plugin for Git {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![
            Self::is_command_in_path("git"),
            Self::is_command_in_path("gh"),
            Self::is_command_in_path("az"),
            Self::git_config_check(),
        ]
    }
}

fn apply_fix_config() -> Result<(), String> {
    let entry = "push.autoSetupRemote";
    let mut config = git2::Config::open_default().expect("git config lookup failed!");

    let result = config.set_bool(entry, true);
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err("git config lookup failed".into()),
    }
}

impl Git {
    fn git_config_check() -> Result<DoctorSuccess, DoctorFailure> {
        let entry = "push.autoSetupRemote";
        let config = git2::Config::open_default().expect("git config lookup failed!");
        match config.get_bool(entry) {
            Ok(c) if c == true => Ok(DoctorSuccess {
                message: format!("{} is configured correct with {} ", entry, c.to_string()),
                plugin: "git - config".into(),
            }),
            Ok(c) if c == false => Err(DoctorFailure {
                message: format!("{} is not configured wrong with {}", entry, c.to_string()),
                plugin: "git - config".into(),
                fix: Some(Box::new(apply_fix_config)),
            }),
            _ => Err(DoctorFailure {
                message: format!("{} is not configured wrong.", entry),
                plugin: "git - config".into(),
                fix: Some(Box::new(apply_fix_config)),
            }),
        }
    }

    fn is_command_in_path(command: &str) -> Result<DoctorSuccess, DoctorFailure> {
        let res = match Command::new(command)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(_) => Ok(DoctorSuccess {
                message: format!("{} is installed", command),
                plugin: command.to_string(),
            }),
            Err(_) => Err(DoctorFailure {
                message: format!(
                    "tool {} is not available. Make sure it is in the PATH",
                    command
                ),
                plugin: command.to_string(),
                fix: Some(Box::new(|| {
                    println!("Please installed");
                    false
                })),
            }),
        };
        res
    }
}
