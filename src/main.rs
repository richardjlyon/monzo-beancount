//! # Monzo Beancount
//!
//! [Monzo bank](https://monzo.com/) offers a feature that allows users to export their transaction data directly to Google Sheets. This
//! integration simplifies the process of tracking and analyzing personal finances by automatically updating
//! a Google Sheet with real-time transaction details. Users can set up this feature through the Monzo app,
//! enabling seamless synchronization of their banking transactions with their spreadsheet.
//!
//! [Beancount](https://beancount.github.io/docs/the_double_entry_counting_method.html) is a text-based double-entry
//! accounting system that facilitates precise tracking of financial
//! transactions using a straightforward and readable format. Each transaction in Beancount involves at least
//! two accounts—one is debited and the other credited—ensuring that the accounting equation (Assets = Liabilities + Equity)
//! is always balanced. The system supports multiple currencies and commodities, and can generate
//! comprehensive financial reports such as balance sheets and income statements. Beancount's plain text
//! files are easy to edit with any text editor, and its functionality can be extended with plugins
//! for custom features or data integration. This makes Beancount ideal for both personal finance management
//! and small business accounting.
//!
//! **Monzo Beancount** is a command line interface app that leverages the Monzo-Google Sheets integration to generate Beancount
//! accounting files from Monzo transaction data. The tool reads transaction details from a Google Sheet,
//! converts them into Beancount transactions, and writes the transactions to a Beancount file. This
//! enables Monzo users to maintain accurate financial records in Beancount format without manual data entry.
//!
//! ## Usage
//! ```shell
//! > monzo-beancount init # initialises the file system in the home directory
//! > cd ~/beancount
//! > monzo-beancount generate # (re)generates the main Beancount file
//! > bean-check main.beancount # checks the file for errors
//! > bean-web main.beancount # starts the web server
//!   (open URL: http://localhost:8080)
//! ```
//!
//! ## Configuration
//! You'll configure the app with a YAML file, specifyinhg the location to store the generated Beancount file and your account information.
//! You can add additional files to track financial information from other sources. Place the `beancount` files in the `include` directory and they
//! will be included.
//!
//! ## Importing Transactions
//! The Monzo API is broken. They provide a 'Pot' facility for separating money with an account. But for reasons known only to them, any transactions made
//! from a pot are not included in transaction data. If you use pots, and you want to use this, you'll need to manually add those transactions.
//! Create a `.csv` file for each pot you want to import with the following fields:
//! ```text
//! date,description,amount,local_currency,local_amount,category
//! 2024-04-14,PATH TAPP PAYGO CP NEW JERSEY USA,-0.8,USD,-1.0,Transport
//! ```
//! Then execute the following command:
//! ```shell
//! > monzo-beancount import
//! ```
//!
//! This will create a `.beancount` file for each `.csv` file, place them in the `include directory`, and include them in the main file.

mod beancount;
mod cli;
mod configuration;
mod error;

use clap::Parser;
use cli::{command, Cli, Commands};
use colored::Colorize;
use configuration::get_configuration;
use error::AppError as Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = get_configuration().expect("Failed to read configuration");

    println!("->> {:#?}", config);

    let bc = match beancount::Beancount::with_data_dir(config.application.data_dir) {
        Ok(b) => b,
        Err(Error::ConfigurationError(_)) => {
            println!(
                "{}",
                "ERROR: Not configured. Run `monzno-beancount init`".red()
            );
            return Err(Error::ApplicationError("Bad Configuration".to_string()));
        }
        Err(e) => return Err(e),
    };

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init {} => match command::init().await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },

        Commands::Generate {} => match command::generate(&bc).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },

        Commands::Sheets {} => match command::sheets(&bc).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },

        Commands::Import {} => match command::import(&bc).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Server { interval_secs } => match command::server(&bc, *interval_secs).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }

    Ok(())
}
