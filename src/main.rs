use clap::{Parser, Subcommand};
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
#[clap(name = "dg cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a SQL server inside a docker container
    Sql(SqlCommand),
    /// Recover the DG CLI
    Fix(FixCommand),
    /// .NET helpers
    Dotnet(DotnetCommand),
    /// Detect and fix problems
    Doctor(DoctorCommand),
    /// Run kubeconfig helpers
    Kubernetes(KubernetesCommand),
    /// Run a MongoDB server inside a docker container
    #[clap(name = "mongodb")]
    MongoDb(MongoDbCommand),
    /// Git helpers
    Git(GitCommand),
    /// GraphQL helpers
    #[clap(name = "graphql")]
    GraphQl(GraphQlCommand),
    Search(SearchCommand),
    Dg(DgCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    config::init_config().await?;

    match cli.command {
        Commands::MongoDb(mongodb_command) => mongo_db::MongoDb::run(mongodb_command),
        Commands::Sql(sql_command) => sql::Sql::run(sql_command).await?,
        Commands::Dotnet(command) => dotnet::Dotnet::run(command).expect("Reason"),
        Commands::Git(git_command) => git::Git::run(git_command),
        Commands::Fix(_) => {
            fix::Fix::run()?;
        }
        Commands::Doctor(dr_command) => og_cli::doctor::run(dr_command),
        Commands::Kubernetes(kubernetes_command) => {
            kube::Kubernetes::run(kubernetes_command).await?
        }
        Commands::GraphQl(graphql_command) => {
            GraphQl::run(graphql_command)?;
        }
        Commands::Search(search_command) => search::Search::run(search_command).await?,
        // default is to forward unknown commands to the python dg cli
        Commands::Dg(dg_command) => {
            DgCli::run(dg_command)?;
        }
    }

    Ok(())
}
