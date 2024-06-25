use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub sql_password: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sql_password: "thats_not_it".into(),
        }
    }
}
