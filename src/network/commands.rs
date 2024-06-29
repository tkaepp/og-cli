use clap::{Args, Subcommand};

#[derive(Args)]
pub struct NetworkCommand {
    #[command(subcommand)]
    // if this is in a seperate file it needs to be public
    pub command: NetworkSubCommands,
}

#[derive(Subcommand)]
pub enum NetworkSubCommands {
    /// Run a network test to validate various connections within the company
    RunTest,
}
