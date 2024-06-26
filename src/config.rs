use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

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
