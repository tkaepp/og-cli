use clap::Args;
use eyre::Result;
use std::process::Command;

/// Passthrough to DG CLI
#[derive(Debug, Args)]
pub struct DgCommand;
// {
//     #[command(subcommand)]
//     command: DgSubCommands,
// }
//
// #[derive(Subcommand, Debug)]
// pub  struct DgSubCommands{
//     plugin { additional_params: Option<String> },
//
// }

pub struct DgCliPlugin;

impl DgCliPlugin {
    pub fn run(_: DgCommand) -> Result<()> {
        let dg_path = "dg";
        Command::new(dg_path)
            .env("DG_CLI_USER_TYPE", "autonomous")
            .args(["--help"])
            .spawn()?;

        Ok(())
    }

    pub fn run2() -> Result<()> {
        let dg_path = "dg";
        Command::new(dg_path)
            .env("DG_CLI_USER_TYPE", "autonomous")
            .args(["--help"])
            .spawn()?;

        Ok(())
    }

    pub fn run_from_plain_args(args: Vec<String>) -> Result<()> {
        let dg_path = "dg";
        Command::new(dg_path)
            .env("DG_CLI_USER_TYPE", "autonomous")
            .args(args)
            .spawn()?;

        Ok(())
    }
}
