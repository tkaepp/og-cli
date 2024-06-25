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
                url
                // azureVaultName,
                // azureSecretName,
            } => {
                // let apiKey = match azureVaultName {
                //     None => "",
                //     Some(valueName) => &valueName,
                // };

                // match azureSecretName.and(azureVaultName) {
                //     Some(tuple) => todo!(),
                //     None => todo!(),
                // }
                // let credential = azure_identity::create_credential().unwrap();
                

                let mut easy = Easy::new();
                let mut list = List::new();

                let mut authorisation = "Authorization: Basic ".to_owned();
                authorisation.push_str(&config.curl_api_key);    
                list.append(&authorisation).unwrap();
                easy.http_headers(list).unwrap();

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
