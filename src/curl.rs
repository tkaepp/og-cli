use azure_security_keyvault::KeyvaultClient;
use std::io::{stdout, Write};

use curl::easy::{Easy, List};

use crate::{config::Curl_Api_Config, plugin::Plugin};
use clap::{Args, Subcommand};

pub struct Curl;

#[derive(Args, Debug)]
pub struct CurlCommand {
    #[command(subcommand)]
    command: CurlSubcommands,
}

#[derive(Subcommand, Debug)]
enum CurlSubcommands {
    Get {
        url: String,
        // azureVaultName: Option<String>,
        // azureSecretName: Option<String>,
    },
}

impl Plugin for Curl {
    fn doctor(&self) {
        println!("TODO potentially ping google to check network");
    }
}

impl Curl {
    pub fn run(cli: CurlCommand, config: Vec<Curl_Api_Config>) {
        match (cli.command) {
            CurlSubcommands::Get {
                url,
            } => {
                let api_config_option = config
                    .iter()
                    .find(|api_config| url.contains(&api_config.curl_base_uri));

                let mut easy = Easy::new();
                let mut list = List::new();

                match api_config_option {
                    Some(api_config) => {
                            println!("{}", api_config.curl_api_key);
                            let mut authorisation = "Authorization: Basic ".to_owned();
                            authorisation.push_str(&api_config.curl_base_uri);
                            list.append(&authorisation).unwrap();
                            easy.http_headers(list).unwrap();
                    }
                    None => {}
                }

                easy.url(&url).unwrap();
                easy.write_function(|data| {
                    stdout().write_all(data).unwrap();
                    Ok(data.len())
                })
                .unwrap();
                easy.perform().unwrap();

                println!("{}", easy.response_code().unwrap());
            }
        }
    }
}

// Query -> https://webapi-oft-search.k8s.devinite.com
// URL -> POD -> Container -> Dg.Application -> Kestrel -> API-Key ()

// Print a web page onto stdout
// fn main() {
//     let mut easy = Easy::new();
//     easy.url("https://www.rust-lang.org/").unwrap();
//     easy.write_function(|data| {
//         stdout().write_all(data).unwrap();
//         Ok(data.len())
//     }).unwrap();
//     easy.perform().unwrap();

//     println!("{}", easy.response_code().unwrap());
// }
