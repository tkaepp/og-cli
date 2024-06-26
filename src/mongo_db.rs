use std::collections::HashMap;

use clap::{Args, Subcommand};

use crate::{common_docker::DockerComposeBuilder, plugin::Plugin};
use crate::common_docker::{DockerCompose, Volume};
use crate::doctor::{DoctorFailure, DoctorSuccess};

pub struct MongoDb;

#[derive(Args, Debug)]
pub struct MongoDbCommand {
    #[command(subcommand)]
    command: MongoDbSubCommands,
}

impl Plugin for MongoDb {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        let mut result = Vec::new();

        if DockerCompose::is_running() {
            result.insert(
                result.len(),
                Ok(DoctorSuccess {
                    message: "Docker daemon is running".into(),
                    plugin: "MongoDB".into(),
                }),
            );
        } else {
            result.insert(
                result.len(),
                Err(DoctorFailure {
                    message: "Docker daemon is not running".into(),
                    plugin: "MongoDB".into(),
                }),
            );
        }

        result
    }
}

impl MongoDb {
    pub fn run(cli: MongoDbCommand) {
        let mongodb_cmd = cli.command;
        let mut environment = HashMap::new();
        environment.insert("MONGO_INITDB_ROOT_USERNAME".into(), "admin".into());
        environment.insert("MONGO_INITDB_ROOT_PASSWORD".into(), "admin".into());

        let mut port_mapping = HashMap::new();
        port_mapping.insert(27017, 27017);

        let mut volumes = Vec::new();
        volumes.insert(
            0,
            Volume {
                volume_name: "mongodb-data".into(),
                volume_type: "volume".into(),
                bind: "/data/db".into(),
                mode: "rw".into(),
            },
        );

        let compose = DockerComposeBuilder::new()
            .add_service(
                "mongodb-local",
                "mongo:latest",
                None,
                Some(environment),
                Some(port_mapping),
                Some(volumes),
            )
            .build();
        match mongodb_cmd {
            MongoDbSubCommands::Start => {
                println!("Starting MongoDB");
                compose.start();
            }
            MongoDbSubCommands::Stop => {
                println!("Stopping MongoDB");
                compose.stop();
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum MongoDbSubCommands {
    Start,
    Stop,
}
