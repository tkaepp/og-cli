use crate::common_docker::DockerComposeBuilder;

use clap::{Args, Subcommand};


#[derive(Args, Debug)]
pub  struct Busybox {
    #[command(subcommand)]
    command: BusyboxSubcommands,
}

#[derive(Subcommand, Debug)]
pub  enum BusyboxSubcommands {
    Start,
    Stop,
}

pub  fn run_busybox(cli: Busybox) {
    let busybox_cmd = cli.command;
    let compose = DockerComposeBuilder::new()
        .add_service("busybox", "busybox:latest", Some("[\"sleep\", \"infinity\"]"))
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
