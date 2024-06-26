use std::collections::HashMap;

use crate::get_config;
use bollard::container::{
    Config, CreateContainerOptions, RestartContainerOptions, StartContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::ContainerStateStatusEnum::{EMPTY, EXITED, RUNNING};
use bollard::models::{ContainerStateStatusEnum, HostConfig, PortBinding};
use bollard::Docker;
use clap::{Args, Subcommand};
use eyre::Result;
use futures_util::TryStreamExt;

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
    pub async fn run(cli: SqlCommand) -> Result<()> {
        let sql_cmd = cli.command;
        let docker = init_docker().await?;
        let status = get_container_status(docker.clone()).await?;

        match sql_cmd {
            SqlSubcommands::Start => {
                if status == RUNNING {
                    println!(
                        "Container {} is already running, nothing to do.",
                        CONTAINER_NAME
                    );
                    return Ok(());
                }
                start(docker, status).await?;
            }
            SqlSubcommands::Stop => {
                if status == EXITED {
                    println!(
                        "Container {} is already stopped, nothing to do.",
                        CONTAINER_NAME
                    );
                    return Ok(());
                }
                stop(docker).await?;
            }
            SqlSubcommands::Remove => {
                if status == EMPTY {
                    println!(
                        "Container {} doesn't exist, nothing to remove.",
                        CONTAINER_NAME
                    );
                    return Ok(());
                }
                if status == RUNNING {
                    println!(
                        "Container {} is running, it must be stopped first.",
                        CONTAINER_NAME
                    );
                    stop(docker.clone()).await?;
                }

                remove(docker).await?;
            }
            SqlSubcommands::Status => {
                println!("Container {} status: {:?}", CONTAINER_NAME, status);
            }
        }

        Ok(())
    }
}

async fn remove(docker: Docker) -> Result<()> {
    println!("Removing container {}...", CONTAINER_NAME);
    docker
        .remove_container(CONTAINER_NAME.as_ref(), None)
        .await?;
    println!("Container {} removed ", CONTAINER_NAME);
    Ok(())
}

async fn start(docker: Docker, status: ContainerStateStatusEnum) -> Result<()> {
    if status == EMPTY {
        println!(
            "Container {} doesn't exist, container will be created and started...",
            CONTAINER_NAME
        );
        create_and_run_container(docker).await?;
        return Ok(());
    }

    println!(
        "Container {} exists but was stopped, container will restart...",
        CONTAINER_NAME
    );
    restart_container(docker).await?;
    Ok(())
}

async fn stop(docker: Docker) -> Result<()> {
    println!("Stopping container {}...", CONTAINER_NAME);
    docker.stop_container(CONTAINER_NAME.as_ref(), None).await?;
    println!("Container {} stopped ", CONTAINER_NAME);
    Ok(())
}

async fn restart_container(docker: Docker) -> Result<()> {
    Ok(docker
        .restart_container(CONTAINER_NAME, Some(RestartContainerOptions { t: 10 }))
        .await?)
}

async fn create_and_run_container(docker: Docker) -> Result<()> {
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

    let image_options = Some(CreateImageOptions {
        from_image: IMAGE_NAME,
        tag: "latest",
        ..Default::default()
    });

    let mut stream = docker.create_image(image_options, None, None);

    while let Some(output) = stream.try_next().await? {
        println!("{:?} - {:?}", output.status, output.progress);
    }

    let result = docker.create_container(options, creation_config).await?;

    docker
        .start_container(CONTAINER_NAME, None::<StartContainerOptions<String>>)
        .await?;

    println!("Container {} created and started", result.id);

    Ok(())
}

async fn get_container_status(docker: Docker) -> Result<ContainerStateStatusEnum> {
    let inspect = docker
        .inspect_container(CONTAINER_NAME.as_ref(), None)
        .await;
    match inspect {
        Ok(i) => Ok(i.state.unwrap().status.unwrap()),
        Err(e) => {
            if e.to_string().contains("404") {
                Ok(EMPTY)
            } else {
                Err(eyre::eyre!(e))
            }
        }
    }
}

async fn init_docker() -> Result<Docker> {
    Ok(Docker::connect_with_local_defaults()?)
}
