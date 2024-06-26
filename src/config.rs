use std::{
    fs::{self, File},
    io::copy,
    sync::OnceLock,
};

use eyre::{eyre, ContextCompat, Result};
use figment::{
    providers::{Format, Json, Serialized},
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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sql_password: "thats_not_it".into(),
            rancher_base_url: "url".into(),
        }
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub async fn init_config() -> Result<()> {
    let og_dir = get_my_home()?
        .context("Could not get home directory")?
        .join(".og-cli");
    fs::create_dir_all(&og_dir)?;

    let ogrc = og_dir.join(".ogrc.json");

    if !ogrc.exists() {
        println!("Config doesn't exist yet, fetching from {CONFIG_URL}");
        let content = reqwest::get(CONFIG_URL).await?.text().await?;
        copy(&mut content.as_bytes(), &mut File::create(&ogrc)?)?;
    }

    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Json::file(&ogrc))
        .extract()?;
    CONFIG
        .set(config)
        .map_err(|_| eyre!("Failed to set config"))?;

    Ok(())
}
