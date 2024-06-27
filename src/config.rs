use clap::ValueEnum;
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

#[derive(Serialize, Deserialize)]
pub struct Search_Api_Config {
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



#[derive(Clone, Debug, ValueEnum)]
pub enum Language {
    LanguageDe,
    LanguageEn,
    LanguageFr,
    LanguageIt,
    LanguageNl,
}

impl Language {
    pub fn get_language_code(&self) -> &str {
        match self {
            Language::LanguageDe => "de-CH",
            Language::LanguageEn => "en-US",
            Language::LanguageFr => "fr-CH",
            Language::LanguageIt => "it-CH",
            Language::LanguageNl => "de-CH",
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Portal {
    PortalChGalaxus,
    PortalChDigitec,
    PortalDe,
    PortalIt,
    PortalFr,
    PortalNl,
    PortalBe,
    PortalAt,
}

impl Portal {
    pub fn get_portal_id(&self) -> i8 {
        match self {
            Portal::PortalChGalaxus => 22,
            Portal::PortalChDigitec => 25,
            Portal::PortalDe => 27,
            Portal::PortalAt => 28,
            Portal::PortalIt => 35,
            Portal::PortalFr => 32,
            Portal::PortalNl => 33,
            Portal::PortalBe => 34,
        }
    }
}
