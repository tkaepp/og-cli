use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct GitCommand {
    #[command(subcommand)]
    command: GitSubCommands,
}


#[derive(Subcommand, Debug)]
pub enum GitSubCommands {
    /// Setup your local git config to use ssh and a seperate .gitconfig for your work projects
    Setup,
}