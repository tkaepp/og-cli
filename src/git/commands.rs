use clap::{Args, Subcommand};

/// Git helpers
#[derive(Args, Debug)]
pub struct GitCommand {
    #[command(subcommand)]
    // if this is in a seperate file it needs to be public
    pub command: GitSubCommands,
}

#[derive(Subcommand, Debug)]
pub enum GitSubCommands {
    /// Setup your local git config to use ssh and a seperate .gitconfig for your work projects
    Setup,
}
