use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use eyre::Result;
use std::{env, process};

#[cfg(feature = "git")]
use og_cli::git::{GitCommand, GitPlugin};
use og_cli::{
    config,
    dg::{DgCliPlugin, DgCommand},
    doctor::{self, DoctorCommand},
    dotnet::{DotnetCommand, DotnetPlugin},
    fix::{self, FixCommand},
    graphql::{GraphQlCommand, GraphQlPlugin},
    kube::{KubernetesCommand, KubernetesPlugin},
    mongo_db::{MongoDbCommand, MongoDbPlugin},
    network::{NetworkCommand, NetworkPlugin},
    search::{SearchCommand, SearchPlugin},
    sql::{SqlCommand, SqlPlugin},
};

#[derive(Parser)]
#[command(version, about)]
#[clap(name = "og cli")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a SQL server inside a docker container
    Sql(SqlCommand),
    /// Run a MongoDB server inside a docker container
    #[clap(name = "mongodb")]
    MongoDb(MongoDbCommand),
    /// GraphQL helpers
    #[clap(name = "graphql")]
    GraphQl(GraphQlCommand),
    /// Access the search API
    Search(SearchCommand),
    /// Detect and fix problems
    Doctor(DoctorCommand),
    /// Recover the DG CLI (currently macOS only)
    #[clap(name = "fix-beta")]
    Fix(FixCommand),
    /// .NET helpers
    #[clap(name = "dotnet-beta")]
    Dotnet(DotnetCommand),
    /// Run kubeconfig helpers (currently Unix only)
    #[clap(name = "kube-beta")]
    Kubernetes(KubernetesCommand),
    /// Git helpers
    #[cfg(feature = "git")]
    #[clap(name = "git-beta")]
    Git(GitCommand),
    #[clap(name = "dg-beta")]
    /// Passthrough to DG CLI
    Dg(DgCommand),
    #[clap(name = "network-beta")]
    /// BETA Run a network validation test
    Network(NetworkCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    config::init_config().await?;

    let cli = Cli::try_parse();

    match cli {
        Ok(c) => {
            match c.command {
                Some(Commands::MongoDb(mongodb_command)) => MongoDbPlugin::run(mongodb_command),
                Some(Commands::Sql(sql_command)) => SqlPlugin::run(sql_command).await?,
                Some(Commands::Dotnet(command)) => DotnetPlugin::run(command).expect("Reason"),
                #[cfg(feature = "git")]
                Some(Commands::Git(git_command)) => GitPlugin::run(git_command),
                Some(Commands::Fix(_)) => {
                    fix::FixPlugin::run()?;
                }
                Some(Commands::Doctor(dr_command)) => doctor::run(dr_command),
                Some(Commands::Kubernetes(kubernetes_command)) => {
                    KubernetesPlugin::run(kubernetes_command).await?
                }
                Some(Commands::GraphQl(graphql_command)) => {
                    GraphQlPlugin::run(graphql_command)?;
                }
                Some(Commands::Search(search_command)) => SearchPlugin::run(search_command).await?, // default is to forward unknown commands to the python dg cli
                Some(Commands::Dg(dg_command)) => {
                    DgCliPlugin::run(dg_command)?;
                }
                Some(Commands::Network(network_command)) => {
                    NetworkPlugin::run(network_command);
                }
                None => {
                    let mut cmd = Cli::command();
                    cmd.build();
                    let _ = cmd.print_help();
                    process::exit(0);
                }
            }
        }
        Err(e) => {
            let args: Vec<String> = env::args().skip(1).collect();

            let mut cmd = Cli::command();
            cmd.build();

            match e.kind() {
                ErrorKind::InvalidValue
                | ErrorKind::UnknownArgument
                | ErrorKind::NoEquals
                | ErrorKind::ValueValidation
                | ErrorKind::TooManyValues
                | ErrorKind::TooFewValues
                | ErrorKind::WrongNumberOfValues
                | ErrorKind::ArgumentConflict
                | ErrorKind::MissingRequiredArgument
                | ErrorKind::MissingSubcommand
                | ErrorKind::InvalidUtf8
                | ErrorKind::DisplayHelp
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                | ErrorKind::DisplayVersion
                | ErrorKind::Io
                | ErrorKind::Format => {
                    e.print()?;
                    std::process::exit(0);
                }
                _ => {
                    DgCliPlugin::run_from_plain_args(args)?;
                }
            }
        }
    }

    Ok(())
}
