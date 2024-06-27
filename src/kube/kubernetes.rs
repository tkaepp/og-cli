use std::fmt::{Display, Formatter};
use std::io::Read;

use clap::{Args, Subcommand};
use colored::Colorize;
use dialoguer::MultiSelect;
use eyre::Context;

use crate::kube::kube_config;
use crate::kube::kube_config::*;
use crate::kube::rancher::*;

pub const KEYRING_SERVICE_ID: &str = "dg_cli_plugin_kube";
pub const KEYRING_KEY: &str = "rancher_token";
pub const RANCHER_CLUSTER_SUFFIX_LENGTH: usize = 3;
pub const RANCHER_CLUSTER_PREFIX: &str = "dg-";

pub struct Kubernetes;

#[derive(Args, Debug)]
pub struct KubernetesCommand {
    #[command(subcommand)]
    command: KubernetesSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum KubernetesSubcommands {
    /// Synchronizes the DG Rancher Kubernetes contexts
    Sync,
    /// Cleanup local kubeconfig
    Cleanup,
}

#[derive(Clone)]
pub struct Cluster {
    pub id: String,
    pub name: String,
    pub name_suffix: String,
    pub server: String,
    pub token_url: Option<String>,
}

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
            self.local_cluster
                .as_ref()
                .map_or_else(|| "NEW".to_string(), get_cluster_fullname),
            self.rancher_cluster
                .as_ref()
                .map_or_else(|| "DELETE".to_string(), get_cluster_fullname),
        )
    }
}

impl Kubernetes {
    pub async fn run(cli: KubernetesCommand) -> eyre::Result<()> {
        match cli.command {
            KubernetesSubcommands::Sync => run_sync()
                .await
                .context("Unable to sync clusters due to errors")?,
            KubernetesSubcommands::Cleanup => {
                run_cleanup().context("Unable to cleanup local kubeconfig due to errors")?
            }
        }

        Ok(())
    }
}

async fn run_sync() -> eyre::Result<()> {
    println!("{}", "Depending on your OS, you have to confirm or enter your password to access the credential store to retrieve the necessary access tokens [ENTER]".bright_green());
    let buffer = &mut [0u8];
    std::io::stdin().read_exact(buffer).unwrap();

    let rancher_token = get_rancher_token()?;
    let rancher_clusters = get_rancher_clusters(&rancher_token).await;
    let local_clusters = get_local_clusters()?;

    if rancher_clusters.is_empty() {
        println!("{}", "No clusters found to sync".red());
        return Ok(());
    }

    println!(
        "Found {} Rancher clusters",
        rancher_clusters.len().to_string().green()
    );

    println!(
        "Found {} local clusters",
        local_clusters.len().to_string().green()
    );

    println!();

    let cluster_synch_actions = get_cluster_sync_actions(local_clusters, rancher_clusters);
    if cluster_synch_actions.is_empty() {
        println!("{}", "Your config is already up to date.".green());

        return Ok(());
    }

    let selected_actions = MultiSelect::new()
        .with_prompt("Select clusters to sync (Ctrl + C to abort)")
        .items(&cluster_synch_actions)
        .interact()
        .unwrap();
    println!();

    if selected_actions.is_empty() {
        println!("{}", "No sync action selected.".red());

        return Ok(());
    }

    for selected_action in selected_actions {
        let action = &cluster_synch_actions[selected_action].action;
        let local_cluster = &cluster_synch_actions[selected_action].local_cluster;
        let remote_cluster = &cluster_synch_actions[selected_action].rancher_cluster;

        match action {
            SyncAction::Create => {
                create_kubeconfig_entry(remote_cluster.as_ref().unwrap(), &rancher_token).await?
            }
            SyncAction::Update => {
                update_kubeconfig_entry(
                    local_cluster.as_ref().unwrap(),
                    remote_cluster.as_ref().unwrap(),
                    &rancher_token,
                )
                .await?
            }
            SyncAction::Delete => delete_kubeconfig_entry(local_cluster.as_ref().unwrap())?,
        }
    }

    Ok(())
}

fn run_cleanup() -> eyre::Result<()> {
    let mut kubeconfig = read_kubeconfig()?;
    let clusters: Vec<String> = kubeconfig
        .clusters
        .iter()
        .map(|c| c.name.to_string())
        .collect();

    let mut selected_clusters = MultiSelect::new()
        .with_prompt("Select clusters to delete (Ctrl + C to abort)")
        .items(&clusters)
        .interact()
        .unwrap();
    println!();

    if selected_clusters.is_empty() {
        println!(
            "{}",
            "There are no clusters found to clean up in your local kubeconfig".green()
        );
    }

    selected_clusters.sort_by(|a, b| b.cmp(a));
    for cluster_index in selected_clusters {
        let cluster_name = &kubeconfig.clusters[cluster_index].name.to_string();

        kubeconfig.clusters.retain(|c| c.name.ne(cluster_name));
        kubeconfig.users.retain(|c| c.name.ne(cluster_name));
        kubeconfig.contexts.retain(|c| c.name.ne(cluster_name));
    }

    write_kubeconfig(kubeconfig)?;
    println!(
        "{}",
        "Your local kubeconfig has been cleaned up successfully".green()
    );

    Ok(())
}

fn get_local_clusters() -> eyre::Result<Vec<Cluster>> {
    let kubeconfig = read_kubeconfig()?;

    let local_clusters = kubeconfig
        .clusters
        .into_iter()
        .map(|c| Cluster {
            id: c.name.clone(),
            name: c.name[..c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH].to_string(),
            name_suffix: c.name[c.name.len() - RANCHER_CLUSTER_SUFFIX_LENGTH..].to_string(),
            server: c.cluster.server,
            token_url: None,
        })
        .collect();

    Ok(local_clusters)
}

fn get_cluster_sync_actions(
    local_clusters: Vec<Cluster>,
    rancher_clusters: Vec<Cluster>,
) -> Vec<ClusterSyncAction> {
    let mut cluster_sync_actions: Vec<ClusterSyncAction> = Vec::new();

    let new_clusters: Vec<&Cluster> = rancher_clusters
        .iter()
        .filter(|rc| !local_clusters.iter().any(|lc| lc.name == rc.name))
        .collect();
    for new_cluster in new_clusters {
        cluster_sync_actions.push(ClusterSyncAction {
            local_cluster: None,
            rancher_cluster: Some(new_cluster.clone()),
            action: SyncAction::Create,
        });
    }

    let update_clusters: Vec<&Cluster> = rancher_clusters
        .iter()
        .filter(|rc| {
            local_clusters.iter().any(|lc| {
                lc.name == rc.name && (lc.name_suffix < rc.name_suffix || lc.server != rc.server)
            })
        })
        .collect();
    for update_cluster in update_clusters {
        let lcs: Vec<&Cluster> = local_clusters
            .iter()
            .filter(|lc| lc.name == update_cluster.name)
            .collect();
        if let Some(&first_local_cluster) = lcs.first() {
            cluster_sync_actions.push(ClusterSyncAction {
                local_cluster: Some(first_local_cluster.clone()),
                rancher_cluster: Some(update_cluster.clone()),
                action: SyncAction::Update,
            });
        }
    }

    let delete_clusters: Vec<&Cluster> = local_clusters
        .iter()
        .filter(|lc| {
            !rancher_clusters.iter().any(|rc| rc.name == lc.name)
                && lc.name.starts_with(RANCHER_CLUSTER_PREFIX)
        })
        .collect();
    for delete_cluster in delete_clusters {
        cluster_sync_actions.push(ClusterSyncAction {
            local_cluster: Some(delete_cluster.clone()),
            rancher_cluster: None,
            action: SyncAction::Delete,
        });
    }

    cluster_sync_actions
}

async fn create_kubeconfig_entry(
    rancher_cluster: &Cluster,
    rancher_token: &String,
) -> eyre::Result<()> {
    let mut kubeconfig = read_kubeconfig()?;
    let name = &get_cluster_fullname(rancher_cluster);
    let token_url = rancher_cluster.token_url.as_ref().expect("");
    let rancher_kubeconfig = get_rancher_kubeconfig(token_url.to_string(), rancher_token).await?;

    kubeconfig.clusters.push(NamedCluster {
        name: name.to_string(),
        cluster: kube_config::Cluster {
            certificate_authority_data: None,
            server: rancher_kubeconfig
                .clusters
                .first()
                .unwrap()
                .cluster
                .server
                .to_string(),
        },
    });

    kubeconfig.contexts.push(NamedContext {
        name: name.to_string(),
        context: Context1 {
            cluster: name.to_string(),
            user: name.to_string(),
            namespace: None,
        },
    });

    kubeconfig.users.push(NamedUser {
        name: name.to_string(),
        user: User {
            token: Some(
                rancher_kubeconfig
                    .users
                    .first()
                    .unwrap()
                    .user
                    .token
                    .as_ref()
                    .unwrap()
                    .to_string(),
            ),
            client_certificate_data: None,
            client_key_data: None,
        },
    });

    write_kubeconfig(kubeconfig)
}

async fn update_kubeconfig_entry(
    local_cluster: &Cluster,
    rancher_cluster: &Cluster,
    rancher_token: &String,
) -> eyre::Result<()> {
    let mut kubeconfig = read_kubeconfig()?;
    let token_url = rancher_cluster.token_url.as_ref().expect("");
    let rancher_kubeconfig = get_rancher_kubeconfig(token_url.to_string(), rancher_token).await?;

    let cluster_pos = kubeconfig
        .clusters
        .iter()
        .position(|c| c.name == local_cluster.id)
        .unwrap();
    kubeconfig.clusters[cluster_pos].cluster.server = rancher_kubeconfig
        .clusters
        .first()
        .unwrap()
        .cluster
        .server
        .to_string();

    let user_pos = kubeconfig
        .users
        .iter()
        .position(|u| u.name == local_cluster.id)
        .unwrap();
    kubeconfig.users[user_pos].user.token = Some(
        rancher_kubeconfig
            .users
            .first()
            .unwrap()
            .user
            .token
            .as_ref()
            .unwrap()
            .to_string(),
    );

    write_kubeconfig(kubeconfig)
}

fn delete_kubeconfig_entry(local_cluster: &Cluster) -> eyre::Result<()> {
    let mut kubeconfig = read_kubeconfig()?;

    kubeconfig
        .clusters
        .retain(|c| c.name != get_cluster_fullname(local_cluster));
    kubeconfig
        .contexts
        .retain(|c| c.name != get_cluster_fullname(local_cluster));
    kubeconfig
        .users
        .retain(|c| c.name != get_cluster_fullname(local_cluster));

    write_kubeconfig(kubeconfig)
}

fn get_cluster_fullname(cluster: &Cluster) -> String {
    format!("{}{}", cluster.name, cluster.name_suffix)
}
