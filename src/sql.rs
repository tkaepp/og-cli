use std::collections::HashMap;

use bollard::container::{
    Config, CreateContainerOptions, RestartContainerOptions, StartContainerOptions,
};
use bollard::models::ContainerStateStatusEnum::{EMPTY, EXITED, RUNNING};
use bollard::models::{ContainerStateStatusEnum, HostConfig, PortBinding};
use bollard::Docker;
use clap::{Args, Subcommand};

use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::get_config;
use crate::plugin::Plugin;

pub struct Sql;

#[derive(Args, Debug)]
pub struct SqlCommand {
    #[command(subcommand)]
    command: SqlSubcommands,
}

impl Plugin for Sql {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        Vec::new()
    }
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
        let config = SqlConfiguration::init_sql_config();
        let docker = Docker::connect_with_local_defaults().unwrap();
        let status = get_container_status(docker.clone(), config.clone()).await;

        match sql_cmd {
            SqlSubcommands::Start => {
                if status == RUNNING {
                    println!(
                        "Container {} is already running, nothing to do.",
                        config.container_name
                    );
                    return;
                }
                start(docker, config, status).await;
            }
            SqlSubcommands::Stop => {
                if status == EXITED {
                    println!(
                        "Container {} is already stopped, nothing to do.",
                        config.container_name
                    );
                    return;
                }
                stop(docker, config).await;
            }
            SqlSubcommands::Remove => {
                if status == EMPTY {
                    println!(
                        "Container {} doesn't exist, nothing to remove.",
                        config.container_name
                    );
                    return;
                }
                remove(docker, config).await;
            }
            SqlSubcommands::Status => {
                println!("Container {} status: {:?}", config.container_name, status)
            }
        }
    }
}

async fn remove(docker: Docker, config: SqlConfiguration) {
    println!("Removing container {}...", config.container_name);
    let _ = docker
        .remove_container(config.container_name.as_ref(), None)
        .await;
    println!("Container {} removed ", config.container_name);
}

async fn start(docker: Docker, config: SqlConfiguration, status: ContainerStateStatusEnum) {
    if status == EMPTY {
        println!(
            "Container {} doesn't exist, container will be created and started...",
            config.container_name
        );
        create_and_run_container(docker, config).await;
        return;
    }

    println!(
        "Container {} exists but was stopped, container will restart...",
        config.container_name
    );
    restart_container(docker, config.container_name.clone()).await;
}

async fn stop(docker: Docker, config: SqlConfiguration) {
    println!("Stopping container {}...", config.container_name);
    let _ = docker
        .stop_container(config.container_name.as_ref(), None)
        .await;
    println!("Container {} stopped ", config.container_name);
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
    let pwd = &get_config().sql_password;
    let formatted_pwd = &format!("MSSQL_SA_PASSWORD={pwd}");
    let env = vec![formatted_pwd.to_string(), "ACCEPT_EULA=Y".to_string()];

    let port_bindings = {
        let mut map = HashMap::new();
        map.insert(
            format!("{}/tcp", config.port),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(config.port.to_string()),
            }]),
        );
        map
    };

    let options = Some(CreateContainerOptions {
        name: config.container_name.clone(),
        platform: None,
    });

    let creation_config = Config {
        image: Some(config.image_name.into_string()),
        env: Some(env),
        host_config: Some(HostConfig {
            binds: Some(vec![config.volume_binding.to_string()]),
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
        .start_container(
            &config.container_name.clone(),
            None::<StartContainerOptions<String>>,
        )
        .await
        .unwrap();
    println!("Container {} created and started", result.id);
}

async fn get_container_status(
    docker: Docker,
    config: SqlConfiguration,
) -> ContainerStateStatusEnum {
    let mut filters = HashMap::new();
    filters.insert("name", vec![config.container_name.as_ref()]);

    let inspect = docker
        .inspect_container(config.container_name.as_ref(), None)
        .await;
    match inspect {
        Ok(_) => inspect.unwrap().state.unwrap().status.unwrap(),
        Err(_) => EMPTY,
    }
}

struct SqlConfiguration {
    container_name: Box<str>,
    volume_binding: Box<str>,
    image_name: Box<str>,
    port: i32,
}

impl Clone for SqlConfiguration {
    fn clone(&self) -> SqlConfiguration {
        SqlConfiguration {
            container_name: self.container_name.clone(),
            volume_binding: self.volume_binding.clone(),
            image_name: self.image_name.clone(),
            port: self.port,
        }
    }
}

impl SqlConfiguration {
    fn init_sql_config() -> SqlConfiguration {
        SqlConfiguration {
            container_name: "mssql-local".into(),
            image_name: "mcr.microsoft.com/azure-sql-edge:latest".into(),
            volume_binding: "sql-data:/var/opt/mssql:rw".into(),
            port: 1433,
        }
    }
}
