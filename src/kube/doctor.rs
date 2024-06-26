use colored::Colorize;
use eyre::Result;
use std::path::Path;

use super::{
    kube_config::{get_kubeconfig_path, read_kubeconfig},
    rancher::get_rancher_token,
    KubernetesPlugin,
};
use crate::{
    doctor::{DoctorFailure, DoctorSuccess},
    plugin::Plugin,
};

const PLUGIN_NAME: &str = "Kubernetes";

impl KubernetesPlugin {
    fn is_kubeconfig_existing() -> Result<DoctorSuccess, DoctorFailure> {
        let path = get_kubeconfig_path().unwrap().path;
        if Path::new(path.as_path()).exists() {
            return Ok(DoctorSuccess {
                message: format!("{}", "kubeconfig has been found".green()),
                plugin: PLUGIN_NAME.to_string(),
            });
        }

        Err(DoctorFailure {
            message: format!(
                "{}",
                "No existing kubeconfig has been found. Please add a valid kubeconfig".red()
            ),
            plugin: PLUGIN_NAME.to_string(),
            fix: None,
        })
    }

    fn is_kubeconfig_valid() -> Result<DoctorSuccess, DoctorFailure> {
        if let Err(error) = read_kubeconfig() {
            return Err(DoctorFailure {
                message: format!(
                    "{}: {}",
                    "Error while parsing the kubeconfig".red(),
                    error.to_string().yellow()
                ),
                plugin: PLUGIN_NAME.to_string(),
                fix: None,
            });
        }

        Ok(DoctorSuccess {
            message: format!("{}", "kubeconfig is valid".green()),
            plugin: PLUGIN_NAME.to_string(),
        })
    }

    fn is_rancher_token_available() -> Result<DoctorSuccess, DoctorFailure> {
        if let Err(error) = get_rancher_token() {
            return Err(DoctorFailure {
                message: format!(
                    "{}: {}",
                    "Unable to retrieve Rancher token from credential store".red(),
                    error.to_string().yellow()
                ),
                plugin: PLUGIN_NAME.to_string(),
                fix: None,
            });
        }

        Ok(DoctorSuccess {
            message: format!("{}", "Rancher token found in credential store".green()),
            plugin: PLUGIN_NAME.to_string(),
        })
    }
}

impl Plugin for KubernetesPlugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![
            Self::is_kubeconfig_existing(),
            Self::is_kubeconfig_valid(),
            Self::is_rancher_token_available(),
        ]
    }
}
