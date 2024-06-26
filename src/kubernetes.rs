use std::fmt::{Display, Formatter};
use crate::{plugin::Plugin};
use clap::{Args, Subcommand};
use dialoguer::Select;
use keyring::{Entry,Result};
use rancher::RancherClient;
use crate::doctor::{DoctorFailure, DoctorSuccess};

const KEYRING_SERVICE_ID: &str = "dg_cli_plugin_kube";
const KEYRING_KEY: &str = "rancher_token";
const RANCHER_MGMT_BASE_URL: &str = "https://kubernetes-management.int.devinite.com";

pub struct Kubernetes;

struct Cluster {
    id: String,
    name: String,
}

impl Display for Cluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

#[derive(Args, Debug)]
pub struct KubernetesCommand {
    #[command(subcommand)]
    command: KubernetesSubcommands,
}

impl Plugin for Kubernetes {
    fn doctor(&self) -> Vec<std::result::Result<DoctorSuccess, DoctorFailure>> {
        Vec::new()
    }
}

impl Kubernetes {
    pub async fn run(cli: KubernetesCommand) {
        match cli.command {
            KubernetesSubcommands::Sync => println!("Doing a sync!"),
            KubernetesSubcommands::Test => run_test().await.expect("Unable to sync clusters due to errors")
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum KubernetesSubcommands {
    /// Synchronizes the DG Rancher Kubernetes contexts
    Sync,
    /// Run random stuff to test
    Test,
}

async fn run_test() -> Result<()> {
    let rancher_token = get_rancher_token()?;
    let clusters = get_rancher_clusters(rancher_token).await;

    if clusters.is_empty() {
        println!("No clusters found to sync");
        return Ok(())
    }

    println!("Found {} clusters", clusters.len());
    for cluster in &clusters {
        println!("Id: {}\tName:{}", cluster.id, cluster.name);
    }

    let _ = Select::new()
        .with_prompt("Select cluster to sync")
        .items(&clusters)
        .interact()
        .unwrap();

    Ok(())
}

fn get_rancher_token() -> Result<String> {
    let entry = Entry::new(KEYRING_SERVICE_ID, KEYRING_KEY)?;
    entry.get_password()
}

async fn get_rancher_clusters(rancher_token: String) -> Vec<Cluster> {
    let rancher_client = RancherClient::new(rancher_token, String::from(RANCHER_MGMT_BASE_URL));
    let clusters_result = rancher_client.clusters().await;

    if let Ok(clusters) = clusters_result {
        let clusters = clusters.data.into_iter().map(|c| Cluster {
            id: c.id,
            name: c.name
        }).collect();

        return clusters;
    }

    Vec::new()
}