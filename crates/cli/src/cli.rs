use crate::commands::Commands;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "corty",
    version,
    about = "Corty - AI Native Code Intelligence Platform",
    long_about = "Corty Code - starts an interactive session by default, use -p/--print for non-interactive output"
)]
pub(crate) struct Cli {
    /// Enable debug mode
    #[arg(short, long)]
    pub debug: bool,

    /// Model for the current session
    #[arg(long)]
    pub model: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
