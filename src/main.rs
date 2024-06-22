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
        Commands::Update {} => match command::update().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Sheets {} => match command::sheets().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }

    Ok(())
}
