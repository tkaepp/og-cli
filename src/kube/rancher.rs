use eyre::eyre;
use keyring::Entry;
use rancher::RancherClient;
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::{self};
use serde_yaml::{self};

use crate::get_config;
use crate::kube::KubeConfig;

#[derive(Deserialize)]
struct GenerateKubeconfigResponse {
    #[serde(rename = "baseType")]
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

pub fn get_rancher_token() -> eyre::Result<String> {
    let entry = Entry::new(
        crate::kube::kubernetes::KEYRING_SERVICE_ID,
        crate::kube::kubernetes::KEYRING_KEY,
    )?;

    Ok(entry.get_password()?)
}

pub async fn get_rancher_clusters(rancher_token: &str) -> Vec<crate::kube::kubernetes::Cluster> {
    let rancher_client = RancherClient::new(
        rancher_token.to_string(),
        String::from(&get_config().rancher_base_url),
    );
    let clusters_result = rancher_client.clusters().await;

    if let Ok(clusters) = clusters_result {
        let clusters = clusters
            .data
            .into_iter()
            .map(|c| crate::kube::kubernetes::Cluster {
                id: c.id,
                name: c.name
                    [..c.name.len() - crate::kube::kubernetes::RANCHER_CLUSTER_SUFFIX_LENGTH]
                    .to_string(),
                name_suffix: c.name
                    [c.name.len() - crate::kube::kubernetes::RANCHER_CLUSTER_SUFFIX_LENGTH..]
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
