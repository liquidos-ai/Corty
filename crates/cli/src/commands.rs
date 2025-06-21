use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Check the health of your Corty Code auto-updater
    Doctor,
    /// Refactor existing code
    Index {
        /// Path to the file or directory to Index
        path: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
        /// Set globally (across all projects)
        #[arg(short, long)]
        global: bool,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
        /// Get global configuration
        #[arg(short, long)]
        global: bool,
    },
    /// List all configuration values
    List {
        /// Show global configuration
        #[arg(short, long)]
        global: bool,
    },
    /// Remove a configuration value
    Remove {
        /// Configuration key
        key: String,
        /// Remove from global configuration
        #[arg(short, long)]
        global: bool,
    },
    /// Reset configuration to defaults
    Reset {
        /// Reset global configuration
        #[arg(short, long)]
        global: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}
