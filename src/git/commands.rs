use std::path::PathBuf;

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
    /// Create .ssh/config entries for your tailscale hosts.
    TailscaleAlias {
        /// username for the ssh connection
        #[arg(short = 'u', long)]
        user: String,
        /// ssh identity file path
        #[arg(short = 'i', long)]
        identity_file: PathBuf,
        /// ssh identity file path
        #[arg(short = 'f', long)]
        tag_filter: Option<String>,
        #[arg(short = 'p', long)]
        hostname_prefix: Option<String>,
    },
}
