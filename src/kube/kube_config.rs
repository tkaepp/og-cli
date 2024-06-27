use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use eyre::{Context, ContextCompat, Result};
use homedir::get_my_home;
use serde::{Deserialize, Serialize};
use serde_yaml::{self, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct KubeConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub clusters: Vec<NamedCluster>,
    pub contexts: Vec<NamedContext>,
    #[serde(rename = "current-context")]
    pub current_context: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Value>,
    pub users: Vec<NamedUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedCluster {
    pub cluster: Cluster,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cluster {
    #[serde(rename = "certificate-authority-data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_authority_data: Option<String>,
    pub server: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedContext {
    pub context: Context1,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context1 {
    pub cluster: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedUser {
    pub name: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "client-certificate-data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_certificate_data: Option<String>,
    #[serde(rename = "client-key-data")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

pub struct KubeconfigPath {
    pub path: PathBuf,
    pub backup_path: PathBuf,
}

pub fn read_kubeconfig() -> Result<KubeConfig> {
    // Define the path to the kube config file
    let kube_config_path = get_kubeconfig_path();

    // Read the existing kube config file
    let kube_config_content = fs::read_to_string(kube_config_path?.path)
        .with_context(|| "A valid kubeconfig must exist")?;

    // Parse the YAML content into a KubeConfig struct
    let kube_config: KubeConfig = serde_yaml::from_str(&kube_config_content)
        .with_context(|| "Found kubeconfig is invalid")?;

    Ok(kube_config)
}

pub fn write_kubeconfig(kube_config: KubeConfig, backup: bool) -> Result<()> {
    // Serialize the updated config back to YAML
    let updated_kube_config_content = serde_yaml::to_string(&kube_config)?;
    let kubeconfig_path = get_kubeconfig_path()?;

    if backup {
        fs::copy(&kubeconfig_path.path, &kubeconfig_path.backup_path)?;
    }

    // Write the updated config back to the file
    fs::write(&kubeconfig_path.path, updated_kube_config_content)?;

    Ok(())
}

pub fn get_kubeconfig_path() -> Result<KubeconfigPath> {
    let path = get_my_home()?
        .context("Could not get home directory")?
        .join(".kube/config");

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let backup_path = get_my_home()?
        .context("Could not get home directory")?
        .join(format!(".kube/config.bak-{}", timestamp));

    Ok(KubeconfigPath { path, backup_path })
}
