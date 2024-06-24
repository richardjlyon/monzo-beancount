mod beancount;
mod cli;
mod error;
mod google;

use clap::Parser;
use cli::{command, Cli, Commands};
use error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {} => match command::generate().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },

        Commands::Sheets {} => match command::sheets().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },

        Commands::Process {} => match command::process().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }

    Ok(())
}
