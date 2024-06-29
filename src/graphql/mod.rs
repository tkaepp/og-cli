use arboard::Clipboard;
use clap::{Args, Subcommand};
use eyre::Result;
use gid::{Gid, Type};
use log::info;

use crate::{
    doctor::{DoctorFailure, DoctorSuccess},
    plugin::Plugin,
};

mod gid;

/// GraphQL helpers
#[derive(Debug, Args)]
pub struct GraphQlCommand {
    #[command(subcommand)]
    command: GraphQlSubcommands,
}

#[derive(Subcommand, Debug)]
enum GraphQlSubcommands {
    /// Encode an ID to a base64 GID and copy it to the clipboard.
    Encode {
        /// Name of the type, e.g. Product.
        name: String,
        /// ID you want to encode, e.g. 1234.
        id: String,
        /// Underlying type of the ID.
        #[arg(short = 't', long = "type")]
        id_type: Option<Type>,
    },
    /// Decode a base64 GID and copy it to the clipboard.
    Decode {
        /// ID you want to decode, e.g. UHJvZHVjdAppMTIzNA==.
        id: String,
    },
}

pub struct GraphQlPlugin;

impl GraphQlPlugin {
    pub fn run(cli: GraphQlCommand) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        match cli.command {
            GraphQlSubcommands::Encode { name, id, id_type } => {
                let gid = Gid::new(name, id, id_type);
                let encoded_gid = gid.to_string();
                clipboard.set_text(&encoded_gid)?;
                info!("{encoded_gid}")
            }
            GraphQlSubcommands::Decode { id } => {
                let gid = Gid::try_from(id)?;
                clipboard.set_text(&gid.id)?;
                info!("{gid:#?}");
            }
        }

        Ok(())
    }
}

impl Plugin for GraphQlPlugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        vec![]
    }
}
