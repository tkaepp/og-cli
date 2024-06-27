use eyre::Result;

pub use network::NetworkCommand;

mod network;

pub fn run(_: NetworkCommand) -> Result<()> {
    println!("Running Network Test");

    Ok(())
}
