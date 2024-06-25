use crate::{plugin::Plugin};
use clap::{Args, Subcommand};
use keyring::{Entry,Result};
use rancher::RancherClient;

const KEYRING_SERVICE_ID: &str = "dg_cli_plugin_kube";
const KEYRING_KEY: &str = "rancher_token";
const RANCHER_MGMT_BASE_URL: &str = "https://kubernetes-management.int.devinite.com";

pub struct Kubernetes;

#[derive(Args, Debug)]
pub struct KubernetesCommand {
    #[command(subcommand)]
    command: KubernetesSubcommands,
}

impl Plugin for Kubernetes {
    fn doctor(&self) {
        println!("Ich bein ein Text");
    }
}

impl Kubernetes {
    pub async fn run(cli: KubernetesCommand) {
        match cli.command {
            KubernetesSubcommands::Sync => println!("Doing a sync!"),
            KubernetesSubcommands::Test => match run_test().await {
                Ok(()) => {},
                Err(error) => panic!("Unable to sync clusters due to errors: {error:?}")
            },
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
    let password = get_rancher_token()?;
    let rancher_client = RancherClient::new(password, String::from(RANCHER_MGMT_BASE_URL));
    let clusters = rancher_client.clusters().await.unwrap().data;

    println!("Found clusters");
    for cluster in clusters {
        println!("Id: {}\tName: {}", cluster.id, cluster.name);
    }

    Ok(())
}

fn get_rancher_token() -> Result<String> {
    let entry = Entry::new(KEYRING_SERVICE_ID, KEYRING_KEY)?;
    let password = entry.get_password()?;

    Ok(password)
}