mod cli;
use crate::cli::Cli;
use clap::Parser;
use color_eyre::Result;
pub(crate) mod commands;
use corty_tui::run_tui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.debug {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    match &cli.command {
        Some(_command) => {
            todo!()
        }
        None => run_tui().await?,
    };

    Ok(())
}
