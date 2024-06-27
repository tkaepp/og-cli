use std::env;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use eyre::Result;

use og_cli::dg::{DgCli, DgCommand};
use og_cli::doctor::DoctorCommand;
use og_cli::dotnet::{self, DotnetCommand};
use og_cli::fix::{self, FixCommand};
use og_cli::git;
use og_cli::git::GitCommand;
use og_cli::graphql::{GraphQl, GraphQlCommand};
use og_cli::kube::{self, KubernetesCommand};
use og_cli::mongo_db::{self, MongoDbCommand};
use og_cli::search::SearchCommand;
use og_cli::sql;
use og_cli::sql::SqlCommand;
use og_cli::{config, search};

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
    #[clap(name = "git-beta")]
    Git(GitCommand),
    #[clap(name = "dg-beta")]
    /// Passthrough to DG CLI
    Dg(DgCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    config::init_config().await?;

    let cli = Cli::try_parse();

    match cli {
        Ok(c) => {
            match c.command {
                Some(Commands::MongoDb(mongodb_command)) => mongo_db::MongoDb::run(mongodb_command),
                Some(Commands::Sql(sql_command)) => sql::Sql::run(sql_command).await?,
                Some(Commands::Dotnet(command)) => dotnet::Dotnet::run(command).expect("Reason"),
                Some(Commands::Git(git_command)) => git::Git::run(git_command),
                Some(Commands::Fix(_)) => {
                    fix::Fix::run()?;
                }
                Some(Commands::Doctor(dr_command)) => og_cli::doctor::run(dr_command),
                Some(Commands::Kubernetes(kubernetes_command)) => {
                    kube::Kubernetes::run(kubernetes_command).await?
                }
                Some(Commands::GraphQl(graphql_command)) => {
                    GraphQl::run(graphql_command)?;
                }
                Some(Commands::Search(search_command)) => {
                    search::Search::run(search_command).await?
                } // default is to forward unknown commands to the python dg cli
                Some(Commands::Dg(dg_command)) => {
                    DgCli::run(dg_command)?;
                }
                None => {
                    let mut cmd = Cli::command();
                    cmd.build();
                    let _ = cmd.print_help();
                    std::process::exit(0);
                }
            }
        }
        Err(e) => {
            let args: Vec<String> = env::args().skip(1).collect();

            let mut cmd = Cli::command();
            cmd.build();

            if e.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                || e.kind() == ErrorKind::DisplayHelp
            {
                e.print()?;
                std::process::exit(0);
            }

            if args.is_empty() {
                e.exit();
            } else if args.len() == 1 && args.iter().any(|x| x == "--help" || x == "-h") {
                let _ = cmd.print_help();
                std::process::exit(0);
            } else if args.len() == 1 && args.iter().any(|x| x == "--version" || x == "-v") {
                println!("{}", cmd.render_long_version());
                std::process::exit(0);
            } else {
                DgCli::run_from_plain_args(args)?;
            }
        }
    }

    Ok(())
}
