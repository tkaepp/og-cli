use std::collections::HashMap;

use bollard::container::{
    Config, CreateContainerOptions, RestartContainerOptions, StartContainerOptions,
};
use bollard::models::ContainerStateStatusEnum::{EMPTY, EXITED, RUNNING};
use bollard::models::{ContainerStateStatusEnum, HostConfig, PortBinding};
use bollard::Docker;
use clap::{Args, Subcommand};

use crate::get_config;

pub struct Sql;

const CONTAINER_NAME: &str = "mssql-local";
const IMAGE_NAME: &str = "mcr.microsoft.com/azure-sql-edge:latest";
const VOLUME_BINDING: &str = "sql-data:/var/opt/mssql:rw";
const PORT: i32 = 1433;

#[derive(Args, Debug)]
pub struct SqlCommand {
    #[command(subcommand)]
    command: SqlSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum SqlSubcommands {
    Start,
    Stop,
    Remove,
    Status,
}

impl Sql {
    pub async fn run(cli: SqlCommand) {
        let sql_cmd = cli.command;
        let docker = Docker::connect_with_local_defaults().unwrap();
        let status = get_container_status(docker.clone()).await;

        match sql_cmd {
            SqlSubcommands::Start => {
                if status == RUNNING {
                    println!(
                        "Container {} is already running, nothing to do.",
                        CONTAINER_NAME
                    );
                    return;
                }
                start(docker, status).await;
            }
            SqlSubcommands::Stop => {
                if status == EXITED {
                    println!(
                        "Container {} is already stopped, nothing to do.",
                        CONTAINER_NAME
                    );
                    return;
                }
                stop(docker).await;
            }
            SqlSubcommands::Remove => {
                if status == EMPTY {
                    println!(
                        "Container {} doesn't exist, nothing to remove.",
                        CONTAINER_NAME
                    );
                    return;
                }
                remove(docker).await;
            }
            SqlSubcommands::Status => {
                println!("Container {} status: {:?}", CONTAINER_NAME, status)
            }
        }
    }
}

async fn remove(docker: Docker) {
    println!("Removing container {}...", CONTAINER_NAME);
    let _ = docker.remove_container(CONTAINER_NAME.as_ref(), None).await;
    println!("Container {} removed ", CONTAINER_NAME);
}

async fn start(docker: Docker, status: ContainerStateStatusEnum) {
    if status == EMPTY {
        println!(
            "Container {} doesn't exist, container will be created and started...",
            CONTAINER_NAME
        );
        create_and_run_container(docker).await;
        return;
    }

    println!(
        "Container {} exists but was stopped, container will restart...",
        CONTAINER_NAME
    );
    restart_container(docker).await;
}

async fn stop(docker: Docker) {
    println!("Stopping container {}...", CONTAINER_NAME);
    let _ = docker.stop_container(CONTAINER_NAME.as_ref(), None).await;
    println!("Container {} stopped ", CONTAINER_NAME);
}

async fn restart_container(docker: Docker) {
    docker
        .restart_container(CONTAINER_NAME, Some(RestartContainerOptions { t: 10 }))
        .await
        .unwrap();
}

async fn create_and_run_container(docker: Docker) {
    let pwd = &get_config().sql_password;
    let formatted_pwd = &format!("MSSQL_SA_PASSWORD={pwd}");
    let env = vec![formatted_pwd, "ACCEPT_EULA=Y"];

    let port_bindings = {
        let mut map = HashMap::new();
        map.insert(
            format!("{}/tcp", PORT),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(PORT.to_string()),
            }]),
        );
        map
    };

    let options = Some(CreateContainerOptions {
        name: CONTAINER_NAME,
        platform: None,
    });

    let creation_config = Config {
        image: Some(IMAGE_NAME),
        env: Some(env),
        host_config: Some(HostConfig {
            binds: Some(vec![VOLUME_BINDING.to_string()]),
            port_bindings: Some(port_bindings),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = docker
        .create_container(options, creation_config)
        .await
        .unwrap();
    docker
        .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
        .await
        .unwrap();
    println!("Container {} created and started", result.id);
}

async fn get_container_status(docker: Docker) -> ContainerStateStatusEnum {
    let mut filters = HashMap::new();
    filters.insert("name", vec![CONTAINER_NAME]);

    let inspect = docker
        .inspect_container(CONTAINER_NAME.as_ref(), None)
        .await;
    match inspect {
        Ok(_) => inspect.unwrap().state.unwrap().status.unwrap(),
        Err(_) => EMPTY,
    }
}
