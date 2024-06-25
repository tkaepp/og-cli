use crate::plugin::Plugin;
use clap::{Args, Subcommand};

pub struct Fix;

#[derive(Args, Debug)]
pub struct FixCommand {
    #[command(subcommand)]
    command: FixSubcommands,
}

impl Fix {
    pub fn run(cli: FixCommand) {
        match cli.command {
            FixSubcommands::Hello { name } => println!("Hi {name}"),
        }
    }
}

impl Plugin for Fix {
    fn doctor(&self) {
        println!("Running the fix doctor");
    }
}

#[derive(Subcommand, Debug)]
enum FixSubcommands {
    Hello { name: String },
}
