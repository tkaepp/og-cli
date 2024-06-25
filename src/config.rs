use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub sql_password: String,
    pub curl_api_config : Vec<Curl_Api_Config>
}

#[derive(Serialize, Deserialize)]
pub struct Curl_Api_Config {
    pub curl_api_key: String,
    pub curl_base_uri: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            sql_password: "thats_not_it".into(),
            curl_api_config: vec![]
        }
    }
}
