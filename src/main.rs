use clap::{Parser, Subcommand};
use figment::providers::{Format, Json, Serialized};
use figment::Figment;
use og_cli::busybox::{self, BusyboxCommand};
use og_cli::config::Config;
use og_cli::curl::{self, CurlCommand};
use og_cli::fix::{self, FixCommand};
use og_cli::kubernetes::{self, KubernetesCommand};
use og_cli::mongo_db::{self, MongoDbCommand};
use og_cli::plugin::Plugin;
use og_cli::search::{self, SearchCommand};

#[derive(Parser, Debug)]
#[command(version, about)]
#[clap(name = "dg cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "busybox")]
    Busybox(BusyboxCommand),
    /// Run an sql server inside a docker container
    Sql,
    Kafka,
    Flink,
    Fix(FixCommand),
    Curl(CurlCommand),
    Search(SearchCommand),
    Doctor,
    /// Run kubernetes config helpers
    Kubernetes(KubernetesCommand),
    MongoDb(MongoDbCommand),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Json::file("config.json"))
        .merge(Json::file("curltest.json"))
        .extract()
        .unwrap();
    println!("SQL container password: {}", config.sql_password);

    match cli.command {
        Commands::Busybox(busybox_command) => busybox::Busybox::run(busybox_command),
        Commands::MongoDb(mongodb_command) => mongo_db::MongoDb::run(mongodb_command),
        Commands::Sql => println!("Sql has not been implemented yet"),
        Commands::Kafka => println!("Kafka has not been implemented yet"),
        Commands::Flink => println!("Flink has not been implemented yet"),
        Commands::Curl(curl_command) => curl::Curl::run(curl_command, config.curl_api_config),
        Commands::Fix(fix_command) => {
            fix::Fix::run(fix_command);
        },
        Commands::Search(search_command) => search::Search::run(search_command).await,
        Commands::Doctor => {
            let plugins: Vec<Box<dyn Plugin>> = vec![
                Box::new(fix::Fix),
                Box::new(busybox::Busybox),
                Box::new(mongo_db::MongoDb),
                Box::new(curl::Curl),
            ];
            for plugin in &plugins {
                plugin.doctor();
            }
        }
        Commands::Kubernetes(kubernetes_command) => {
            kubernetes::Kubernetes::run(kubernetes_command).await
        }
    }
}
