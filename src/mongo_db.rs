use std::collections::HashMap;
use crate::{common_docker::DockerComposeBuilder, plugin::Plugin};

use clap::{Args, Subcommand};

pub struct MongoDb;

#[derive(Args, Debug)]
pub struct MongoDbCommand {
    #[command(subcommand)]
    command: MongoDbSubCommands,
}

impl Plugin for MongoDb {
    fn doctor(&self) {
        println!("Running the MongoDB doctor");
    }
}

impl MongoDb {
    pub fn run(cli: MongoDbCommand) {
        let mongodb_cmd = cli.command;
        let mut environment = HashMap::new();
        environment.insert(String::from("MONGO_INITDB_ROOT_USERNAME"), String::from("admin"));
        environment.insert(String::from("MONGO_INITDB_ROOT_PASSWORD"), String::from("admin"));

        let mut port_mapping = HashMap::new();
        port_mapping.insert(27017, 27017);

        let compose = DockerComposeBuilder::new()
            .add_service(
                "mongodb-local",
                "mongo:latest",
                None,
                Some(environment),
                Some(port_mapping)
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
