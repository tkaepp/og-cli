use clap::{Parser, Subcommand};
use og_cli::busybox::{self, BusyboxCommand};
use og_cli::fix::{self, FixCommand};
use og_cli::plugin::Plugin;

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
    Busybox(BusyboxCommand),
    /// Run an sql server inside a docker container
    Sql,
    Kafka,
    Flink,
    Fix(FixCommand),
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Busybox(busybox) => busybox::run_busybox(busybox),
        Commands::Sql => println!("Sql has not been implemented yet"),
        Commands::Kafka => println!("Kafka has not been implemented yet"),
        Commands::Flink => println!("Flink has not been implemented yet"),
        Commands::Fix(fix_command) => {
            fix::Fix::run(fix_command);
        }
        Commands::Doctor => {
            let plugins: Vec<Box<dyn Plugin>> = vec![Box::new(fix::Fix)];
            for plugin in &plugins {
                plugin.doctor();
            }
        }
    }
}
