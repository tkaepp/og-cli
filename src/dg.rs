use std::process::Command;

use clap::Args;
use eyre::Result;

pub struct DgCli;

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

impl DgCli {
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
