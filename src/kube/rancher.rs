use colored::Colorize;
use dialoguer::{Password, Select};
use eyre::eyre;
use keyring::Entry;
use log::info;
use rancher::RancherClient;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::ffi::OsStr;

use super::{
    kubernetes::{self, Cluster},
    KubeConfig,
};
use crate::get_config;

#[derive(Deserialize)]

struct GenerateKubeconfigResponse {
    config: String,
}

pub async fn get_rancher_kubeconfig(
    generate_kubeconfig_url: String,
    rancher_token: &String,
) -> eyre::Result<KubeConfig> {
    let response = Client::new()
        .post(Url::parse(&generate_kubeconfig_url)?)
        .bearer_auth(rancher_token)
        .send()
        .await?;
    let status = response.status();
    let bytes = response.bytes().await?;
    let response_string = String::from_utf8(bytes.to_vec())?;

    if !status.is_success() {
        return Err(eyre!("http {:#?}: {}", status, response_string));
    }

    let json: GenerateKubeconfigResponse = serde_json::from_str(&response_string)?;
    let kubeconfig: KubeConfig = serde_yaml::from_str(&json.config)?;

    Ok(kubeconfig)
}

#[cfg(target_family = "unix")]
pub fn get_rancher_token() -> eyre::Result<String> {
    let entry = Entry::new(kubernetes::KEYRING_SERVICE_ID, kubernetes::KEYRING_KEY)?;

    Ok(entry.get_password()?)
}

#[cfg(target_family = "windows")]
pub fn get_rancher_token() -> eyre::Result<String> {
    let entry = Entry::new_with_target(
        kubernetes::KEYRING_SERVICE_ID,
        kubernetes::KEYRING_SERVICE_ID,
        kubernetes::KEYRING_KEY,
    )?;

    Ok(entry.get_password()?)
}

pub async fn get_rancher_clusters(rancher_token: &str) -> Vec<Cluster> {
    let rancher_client = RancherClient::new(
        rancher_token.to_string(),
        String::from(&get_config().rancher_base_url),
    );
    let clusters_result = rancher_client.clusters().await;

    if let Ok(clusters) = clusters_result {
        let clusters = clusters
            .data
            .into_iter()
            .map(|c| Cluster {
                id: c.id,
                name: c.name[..c.name.len() - kubernetes::RANCHER_CLUSTER_SUFFIX_LENGTH]
                    .to_string(),
                name_suffix: c.name[c.name.len() - kubernetes::RANCHER_CLUSTER_SUFFIX_LENGTH..]
                    .to_string(),
                server: c
                    .links
                    .get("self")
                    .unwrap()
                    .replace("v3", "k8s")
                    .to_string(),
                token_url: Some(c.actions.get("generateKubeconfig").unwrap().to_string()),
            })
            .collect();

        return clusters;
    }

    Vec::new()
}

pub fn add_rancher_token() -> eyre::Result<()> {
    let selected_option = Select::new()
        .with_prompt("Would you like to create a new Rancher API token or use an existing one")
        .default(0)
        .items(&["New Token", "Existing Token"])
        .interact()
        .unwrap();

    match selected_option {
        0 => create_new_rancher_token()?,
        1 => add_existing_rancher_token()?,
        _ => unimplemented!(),
    };

    Ok(())
}

fn create_new_rancher_token() -> eyre::Result<()> {
    let mut rancher_url = get_config().rancher_base_url.clone();
    rancher_url.push_str("/dashboard/account/create-key");

    info!("Use these options to create a new Rancher API token:");
    info!("1. Open {}", &rancher_url);
    info!("2. Create a new Rancher API token");
    info!("\t{}:\t\t{}", "Description".cyan(), "OG-CLI");
    info!("\t{}:\t\t\t{}", "Scope".cyan(), "No Scope");
    info!("\t{}:\t{}", "Automatically expire".cyan(), "Never");

    open::that(OsStr::new(&rancher_url))?;

    add_existing_rancher_token()?;

    Ok(())
}

fn add_existing_rancher_token() -> eyre::Result<()> {
    let token = Password::new()
        .with_prompt("Please enter your Rancher API token")
        .validate_with(|t: &String| -> Result<(), &str> {
            if t.starts_with("token-") {
                Ok(())
            } else {
                Err("Entered token format seems to be invalid. Token must start with 'token-'")
            }
        })
        .interact()
        .unwrap();

    set_rancher_token(&token)?;
    info!(
        "{}",
        "Rancher API token persisted in system credential store".green()
    );

    Ok(())
}

#[cfg(target_family = "unix")]
pub fn set_rancher_token(token: &String) -> eyre::Result<()> {
    let entry = Entry::new(kubernetes::KEYRING_SERVICE_ID, kubernetes::KEYRING_KEY)?;

    Ok(entry.set_password(token)?)
}

#[cfg(target_family = "windows")]
pub fn set_rancher_token(token: &String) -> eyre::Result<()> {
    let entry = Entry::new_with_target(
        kubernetes::KEYRING_SERVICE_ID,
        kubernetes::KEYRING_SERVICE_ID,
        kubernetes::KEYRING_KEY,
    )?;

    Ok(entry.set_password(token1)?)
}
