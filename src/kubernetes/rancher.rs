use eyre::eyre;
use reqwest::{Client, Url};
use crate::kubernetes::KubeConfig;
use serde::{Deserialize};
use serde_yaml::{self};
use serde_json::{self};

#[derive(Debug, Deserialize)]
struct GenerateKubeconfigResponse {
    #[serde(rename = "baseType")]
    base_type: String,
    config: String,
    r#type: String,
}

pub async fn get_rancher_kubeconfig(generate_kubeconfig_url: String, rancher_token: &String) -> eyre::Result<KubeConfig>
{
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