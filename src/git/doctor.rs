use std::convert::Into;
use std::process::Command;

use git2::Error;

use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::git::Git;
use crate::plugin::{DoctorFix, Plugin};

const GIT_CONFIG: &str = "git - config";

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

impl DoctorFix for Git {
    fn apply_fix(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![Self::gitconfig_fixes()]
    }
}

impl From<git2::Error> for DoctorFailure {
    fn from(_: Error) -> Self {
        DoctorFailure {
            message: "git error".into(),
            plugin: GIT_CONFIG.to_string(),
        }
    }
}
impl From<git2::Error> for DoctorSuccess {
    fn from(_: Error) -> Self {
        DoctorSuccess {
            message: "git success".into(),
            plugin: GIT_CONFIG.to_string(),
        }
    }
}

impl Git {
    fn git_config_check() -> Result<DoctorSuccess, DoctorFailure> {
        let entry = "push.autoSetupRemote";
        let config = git2::Config::open_default().expect("git config lookup failed!");
        match config.get_bool(entry) {
            Ok(c) if c == true => Ok(DoctorSuccess {
                message: format!("{} is configured correct with {} ", entry, c.to_string()),
                plugin: GIT_CONFIG.to_string(),
            }),
            Ok(c) if c == false => Err(DoctorFailure {
                message: format!("{} is not configured wrong with {}", entry, c.to_string()),
                plugin: GIT_CONFIG.to_string(),
            }),
            _ => Err(DoctorFailure {
                message: format!("{} is not configured wrong.", entry),
                plugin: GIT_CONFIG.to_string(),
            }),
        }
    }

    fn gitconfig_fixes() -> Result<DoctorSuccess, DoctorFailure> {
        let desired_config = [("push.autoSetupRemote", "true")];
        let mut config = git2::Config::open_default().expect("git config lookup failed!");
        let mut results = vec![];
        let mut errs = vec![];

        for (name, value) in desired_config {
            results.push(config.set_str(name, value));
            errs.extend(config.set_str(name, value).err());
        }

        if errs.is_empty() {
            Ok(DoctorSuccess {
                message: "alskdfj".into(),
                plugin: GIT_CONFIG.to_string(),
            })
        } else {
            Err(DoctorFailure {
                message: "alskdfj".into(),
                plugin: GIT_CONFIG.to_string(),
            })
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
            }),
        };
        res
    }
}
