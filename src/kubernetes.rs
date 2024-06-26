use std::fmt::{Display, Formatter};
use crate::{get_config, plugin::Plugin};
use clap::{Args, Subcommand};
use dialoguer::{MultiSelect};
use keyring::{Entry, Result};
use kube::config::Kubeconfig;
use rancher::RancherClient;
use crate::doctor::{DoctorFailure, DoctorSuccess};

const KEYRING_SERVICE_ID: &str = "dg_cli_plugin_kube";
const KEYRING_KEY: &str = "rancher_token";
const RANCHER_CLUSTER_SUFFIX_LENGTH: usize = 3;

pub struct Kubernetes;

#[derive(Clone, Debug)]
struct Cluster {
    id: String,
    name: String,
    name_suffix: String,
    server: String,
}

// impl Display for Cluster {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{} ({})", self.name, self.id)
//     }
// }

#[derive(Debug)]
enum SyncAction {
    Create,
    Update,
    Delete,
}

struct ClusterSyncAction {
    local_cluster: Option<Cluster>,
    rancher_cluster: Option<Cluster>,
    action: SyncAction,
}

impl Display for ClusterSyncAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {} -> {}",
            self.action,
            self.local_cluster.as_ref().map_or_else(|| "NEW".to_string(), get_cluster_fullname),
            self.rancher_cluster.as_ref().map_or_else(|| "DELETE".to_string(), get_cluster_fullname),
        )
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
            KubernetesSubcommands::Sync => run_sync().await.expect("Unable to sync clusters due to errors"),
            KubernetesSubcommands::Test => {
                let test = get_local_clusters();
                for cluster in test.iter() {
                    println!("Id: {}\tName:{}", cluster.id, cluster.name);
                }
            }
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

async fn run_test() {}

async fn run_sync() -> Result<()> {
    let rancher_token = get_rancher_token()?;
    let rancher_clusters = get_rancher_clusters(rancher_token).await;
    let local_clusters = get_local_clusters();

    if rancher_clusters.is_empty() {
        println!("No clusters found to sync");
        return Ok(());
    }

    println!("Found {} Rancher clusters", rancher_clusters.len());
    for cluster in &rancher_clusters {
        println!("Id: {}\tName:{}", cluster.id, cluster.name);
    }
    println!();


    println!("Found {} local clusters", local_clusters.len());
    for cluster in &local_clusters {
        println!("Id: {}\tName:{}", cluster.id, cluster.name);
    }
    println!();

    let cluster_synch_actions = get_cluster_sync_actions(local_clusters, rancher_clusters);
    let selected_actions = MultiSelect::new()
        .with_prompt("Select cluster to sync")
        .items(&cluster_synch_actions)
        .interact()
        .unwrap();

    println!("Selected indices: {:?}", selected_actions);

    Ok(())
}

fn get_rancher_token() -> Result<String> {
    let entry = Entry::new(KEYRING_SERVICE_ID, KEYRING_KEY)?;
    entry.get_password()
}

async fn get_rancher_clusters(rancher_token: String) -> Vec<Cluster> {
    let rancher_client = RancherClient::new(rancher_token, String::from(&get_config().rancher_base_url));
    let clusters_result = rancher_client.clusters().await;

    if let Ok(clusters) = clusters_result {
        let clusters = clusters.data.into_iter().map(|c| Cluster {
            id: c.id,
            name: c.name[..c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH].to_string(),
            name_suffix: c.name[c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH..].to_string(),
            server: c.links.get("self").unwrap().to_string(),
        }).collect();

        return clusters;
    }

    Vec::new()
}

fn get_local_clusters() -> Vec<Cluster> {
    let kubeconfig_result = Kubeconfig::read();
    if let Ok(kubeconfig) = kubeconfig_result {
        let local_clusters = kubeconfig.clusters.into_iter().map(|c| {
            Cluster {
                id: c.name.clone(),
                name: c.name[..c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH].to_string(),
                name_suffix: c.name[c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH..].to_string(),
                server: c.cluster.unwrap().server.unwrap(),
            }
        }).collect();

        return local_clusters;
    }

    Vec::new()
}

fn get_cluster_sync_actions(local_clusters: Vec<Cluster>, rancher_clusters: Vec<Cluster>) -> Vec<ClusterSyncAction> {
    let mut cluster_sync_actions: Vec<ClusterSyncAction> = Vec::new();

    let new_clusters: Vec<&Cluster> = rancher_clusters.iter().filter(|rc| {
        !local_clusters.iter().any(|lc| lc.name == rc.name)
    }).collect();
    for new_cluster in new_clusters {
        cluster_sync_actions.push(ClusterSyncAction {
            local_cluster: None,
            rancher_cluster: Some(new_cluster.clone()),
            action: SyncAction::Create,
        });
    }

    let update_clusters: Vec<&Cluster> = rancher_clusters.iter().filter(|rc| {
        local_clusters.iter().any(|lc| lc.name == rc.name && lc.name_suffix < rc.name_suffix)
    }).collect();
    for update_cluster in update_clusters {
        let lcs: Vec<&Cluster> = local_clusters.iter().filter(|lc| lc.name == update_cluster.name).collect();
        if let Some(&first_local_cluster) = lcs.first() {
            cluster_sync_actions.push(ClusterSyncAction {
                local_cluster: Some(first_local_cluster.clone()),
                rancher_cluster: Some(update_cluster.clone()),
                action: SyncAction::Update,
            });
        }
    }

    cluster_sync_actions
}

fn get_cluster_fullname(cluster: &Cluster) -> String {
    format!("{}{}", cluster.name, cluster.name_suffix)
}
