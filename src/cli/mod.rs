//! The command line interface.

pub mod command;

use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Contains the commands.
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
/// Available commands.
pub enum Commands {
    /// Initialise the apo
    Init {},
    /// Generate beancount files
    Generate {},
    /// List sheet names
    Sheets {},
    /// Import CSV files
    Import {},
    /// Server
    Server {
        /// Interval in seconds
        #[arg(short, long, default_value_t = 15)]
        interval_secs: u64,
    },
}
