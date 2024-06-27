use crate::{common_docker::DockerComposeBuilder, plugin::Plugin};

use crate::common_docker::DockerCompose;
use crate::doctor::{DoctorFailure, DoctorSuccess};
use clap::{Args, Subcommand};

pub struct Busybox;

#[derive(Args, Debug)]
pub struct BusyboxCommand {
    #[command(subcommand)]
    command: BusyboxSubcommands,
}

impl Plugin for Busybox {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        let mut result = Vec::new();

        if DockerCompose::is_running() {
            result.insert(
                result.len(),
                Ok(DoctorSuccess {
                    message: "Docker daemon is running".into(),
                    plugin: "Busybox".into(),
                }),
            );
        } else {
            result.insert(
                result.len(),
                Err(DoctorFailure {
                    message: "Docker daemon is not running".into(),
                    plugin: "Busybox".into(),
                }),
            );
        }

        result
    }
}

impl Busybox {
    pub fn run(cli: BusyboxCommand) {
        let busybox_cmd = cli.command;
        let compose = DockerComposeBuilder::new()
            .add_service(
                "busybox",
                "busybox:latest",
                Some("[\"sleep\", \"infinity\"]"),
                None,
                None,
                None,
            )
            .build();
        match busybox_cmd {
            BusyboxSubcommands::Start => {
                println!("Starting busybox");
                compose.start();
            }
            BusyboxSubcommands::Stop => {
                println!("Stopping busybox");
                compose.stop();
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum BusyboxSubcommands {
    Start,
    Stop,
}
