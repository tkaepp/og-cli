use std::process::Command;

use clap::Args;
use eyre::Result;

pub struct DgCli;

#[derive(Debug, Args)]
pub struct DgCommand;

impl DgCli {
    pub fn run(_: DgCommand) -> Result<()> {
        let dg_path = "/usr/bin/dg";
        println!("al;ksdjflaskdf");
        Command::new(dg_path)
            .env("DG_CLI_USER_TYPE", "autonomous")
            .args(["--help"])
            .spawn()?;

        Ok(())
    }
}
