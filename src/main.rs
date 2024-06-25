use clap::{Parser, Subcommand};

use busybox::run_busybox;

use crate::busybox::Busybox;

mod common_docker;
mod busybox;

#[derive(Parser, Debug)]
#[command(version, about)]
#[clap(name = "dg cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(name = "busybox")]
    Busybox(Busybox),
    /// Run an sql server inside a docker container
    Sql,
    Kafka,
    Flink,
}


fn main() {
    let cli = Cli::parse();
    dbg!(&cli);

    match cli.command {
        Commands::Busybox(busybox) => run_busybox(busybox),
        Commands::Sql => println!("Sql has not been implemented yet"),
        Commands::Kafka => println!("Kafka has not been implemented yet"),
        Commands::Flink => println!("Flink has not been implemented yet"),
    }
}

