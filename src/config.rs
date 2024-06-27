use std::{
    fs::{self, File},
    io::copy,
    path::Path,
    sync::OnceLock,
};

use eyre::{eyre, Context, ContextCompat, Result};
use figment::{
    providers::{Format, Json},
    Figment,
};
use homedir::get_my_home;
use serde::{Deserialize, Serialize};

const CONFIG_URL: &str =
    "https://dg-package-repositories.platform.test.int.devinite.com/og-cli/config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub sql_password: String,
    pub rancher_base_url: String,
    pub search_urls: SearchUrl
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUrl {
    pub test: String,
    pub prod: String,
    pub oft: String,
}

pub async fn init_config() -> Result<()> {
    let og_dir = get_my_home()?
        .context("Could not get home directory")?
        .join(".og-cli");
    fs::create_dir_all(&og_dir)?;

    let ogrc = og_dir.join(".ogrc.json");

    if !ogrc.exists() {
        println!("Config doesn't exist yet, fetching from {CONFIG_URL}");
        download_config(&ogrc)
            .await
            .context("Unable to fetch config, are you connected to the VPN?")?;
    }

    let config: Config = Figment::new().merge(Json::file(&ogrc)).extract()?;
    CONFIG
        .set(config)
        .map_err(|_| eyre!("Failed to set config"))?;

    Ok(())
}

async fn download_config(destination: &Path) -> Result<()> {
    let content = reqwest::get(CONFIG_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;
    copy(&mut content.as_bytes(), &mut File::create(destination)?)?;

    Ok(())
}

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();
