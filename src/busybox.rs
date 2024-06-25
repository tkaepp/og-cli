use crate::{common_docker::DockerComposeBuilder, plugin::Plugin};

use clap::{Args, Subcommand};

pub struct Busybox;

#[derive(Args, Debug)]
pub struct BusyboxCommand {
    #[command(subcommand)]
    command: BusyboxSubcommands,
}

impl Plugin for Busybox {
    fn doctor(&self) {
        println!("Running the busybox doctor");
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
                None
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
