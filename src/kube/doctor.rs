use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::kube::kube_config::{get_kubeconfig_path, read_kubeconfig};
use crate::kube::rancher::get_rancher_token;
use crate::kube::Kubernetes;
use crate::plugin::Plugin;
use colored::Colorize;
use std::path::Path;

const PLUGIN_NAME: &str = "Kubernetes";

impl Plugin for Kubernetes {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![
            Self::is_kubeconfig_existing(),
            Self::is_kubeconfig_valid(),
            Self::is_rancher_token_available(),
        ]
    }
}

impl Kubernetes {
    fn is_kubeconfig_existing() -> Result<DoctorSuccess, DoctorFailure> {
        let path = get_kubeconfig_path();
        if Path::new(path.unwrap().as_path()).exists() {
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
            });
        }

        Ok(DoctorSuccess {
            message: format!("{}", "Rancher token found in credential store".green()),
            plugin: PLUGIN_NAME.to_string(),
        })
    }
}
