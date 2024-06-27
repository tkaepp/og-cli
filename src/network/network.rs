use crate::network::NetworkCommand;

pub struct Network;

impl Network {
    pub fn run(_: NetworkCommand) -> eyre::Result<()> {
        println!("Running Network Test");

        Ok(())
    }
}
