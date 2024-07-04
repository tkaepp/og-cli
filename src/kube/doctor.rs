use colored::Colorize;
use eyre::Result;
use log::info;

use super::{
    kube_config::{create_empty_kubeconfig, read_kubeconfig},
    rancher::{add_rancher_token, get_rancher_token},
    KubernetesPlugin,
};
use crate::{
    doctor::{DoctorFailure, DoctorSuccess},
    plugin::Plugin,
};

const PLUGIN_NAME: &str = "Kubernetes";

impl KubernetesPlugin {
    fn is_kubeconfig_valid(&self) -> Result<DoctorSuccess, DoctorFailure> {
        if let Err(error) = read_kubeconfig() {
            return Err(DoctorFailure {
                message: format!(
                    "{}: {}",
                    "Error while parsing the kubeconfig".red(),
                    error.to_string().yellow()
                ),
                plugin: PLUGIN_NAME.to_string(),
                fix: Some(Box::new(Self::apply_kubeconfig_fix)),
            });
        }

        Ok(DoctorSuccess {
            message: format!("{}", "kubeconfig is valid".green()),
            plugin: PLUGIN_NAME.to_string(),
        })
    }

    fn is_rancher_token_available(&self) -> Result<DoctorSuccess, DoctorFailure> {
        print_credential_store_warning();

        if let Err(error) = get_rancher_token() {
            return Err(DoctorFailure {
                message: format!(
                    "{}: {}",
                    "Unable to retrieve Rancher token from credential store".red(),
                    error.to_string().yellow()
                ),
                plugin: PLUGIN_NAME.to_string(),
                fix: Some(Box::new(Self::apply_rancher_token_fix)),
            });
        }

        Ok(DoctorSuccess {
            message: format!("{}", "Rancher token found in credential store".green()),
            plugin: PLUGIN_NAME.to_string(),
        })
    }

    fn apply_kubeconfig_fix() -> Result<(), String> {
        create_empty_kubeconfig(true)
            .map_err(|_| format!("{}", "Unable to create a new empty kubeconfig!".red()))
    }

    fn apply_rancher_token_fix() -> Result<(), String> {
        add_rancher_token().map_err(|_| format!("{}", "Unable to add new Rancher API token!".red()))
    }
}

impl Plugin for KubernetesPlugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![
            self.is_kubeconfig_valid(),
            self.is_rancher_token_available(),
        ]
    }
}

fn print_credential_store_warning() {
    info!("Depending on your OS, you have to confirm or enter your password to access the credential store to retrieve the necessary access tokens\n");
}
