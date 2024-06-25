use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct FixArgs {
    #[command(subcommand)]
    command: FixSubcommands,
}

#[derive(Subcommand, Debug)]
enum FixSubcommands {
    Hello { name: String },
}

pub fn run_fix(cli: FixArgs) {
    match cli.command {
        FixSubcommands::Hello { name } => println!("Hi {name}"),
    }
}
