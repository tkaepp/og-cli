use crate::plugin::Plugin;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RestartContainerOptions,
};
use bollard::Docker;
use clap::{Args, Subcommand};
use std::collections::HashMap;
use crate::CONFIG;

pub struct Sql;

#[derive(Args, Debug)]
pub struct SqlCommand {
    #[command(subcommand)]
    command: SqlSubcommands,
}

impl Plugin for Sql {
    fn doctor(&self) {
        println!("Running the Sql doctor");
    }
}
struct SqlConfiguration {
    container_name: Box<str>,
    volume_name: Box<str>,
}

impl SqlConfiguration {
    fn init_sql_config() -> SqlConfiguration {
        SqlConfiguration {
            container_name: "mssql-local".into(),
            volume_name: "sql-data".into(),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum SqlSubcommands {
    Start,
    Stop,
}

impl Sql {
    pub async fn run(cli: SqlCommand) {
        let sql_cmd = cli.command;
        match sql_cmd {
            SqlSubcommands::Start => {
                println!("Starting Sql");
                start().await;
            }
            SqlSubcommands::Stop => {
                println!("Stopping Sql");
            }
        }
    }
}

async fn start() {
    let config = SqlConfiguration::init_sql_config();
    let docker = Docker::connect_with_local_defaults().unwrap();
    let mut filters = HashMap::new();
    filters.insert("name", vec![config.container_name.as_ref()]);

    let options = Some(ListContainersOptions {
        all: true, // This will only return running container
        filters,
        ..Default::default()
    });
    let containers = docker.list_containers(options).await.unwrap();

    if containers
        .iter()
        .any(|c| c.state == Some(String::from("running")))
    {
        println!(
            "Container {} is already running, nothing to do.",
            config.container_name
        );
        return;
    }

    if containers.is_empty() {
        println!(
            "Container {} doesn't exist, container will be created and started...",
            config.container_name
        );
        create_and_run_container(docker, config).await;
        return;
    }

    if containers
        .iter()
        .any(|c| c.state == Some(String::from("exited")))
    {
        println!(
            "Container {} exists but was stopped, container will restart...",
            config.container_name
        );
        restart_container(docker, config.container_name.clone()).await;
    }
}

async fn restart_container(docker: Docker, container_name: Box<str>) {
    docker
        .restart_container(
            container_name.as_ref(),
            Some(RestartContainerOptions { t: 10 }),
        )
        .await
        .unwrap();
}

async fn create_and_run_container(docker: Docker, config: SqlConfiguration) {
    let pwd = &CONFIG.get().unwrap().sql_password;
    let formatted_pwd = &format!("MSSQL_SA_PASSWORD={pwd}");
    let env = vec![formatted_pwd, "ACCEPT_EULA=Y"];

    let options = Some(CreateContainerOptions {
        name: config.container_name.clone(),
        platform: None,
    });

    let config = Config {
        image: Some("mcr.microsoft.com/azure-sql-edge:latest"),
        env: Some(env),
        ..Default::default()
    };

    let result = docker.create_container(options, config).await.unwrap();
    println!("Container {} created and started", result.id);
}
