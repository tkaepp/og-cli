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
        ]
    }
}

impl Git {
    fn is_command_in_path(command: &str) -> Result<DoctorSuccess, DoctorFailure> {
        let res = match Command::new(command)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn() {
            Ok(_) => Ok(DoctorSuccess {
                message: format!("{} is installed", command),
                plugin: command.to_string(),
            }),
            Err(_) => Err(DoctorFailure {
                message: format!("tool {} is not available. Make sure it is in the PATH", command),
                plugin: command.to_string(),
            }),
        };
        res
    }
}
