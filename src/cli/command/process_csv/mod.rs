//! Process CSV files command.
//!
//! Process a set of CSV files and generate Beancount directives. The CSV files are expected to be
//! in a specific format, with the following columns:
//! - date: the date of the transaction
//! - description: a description of the transaction
//! - amount: the amount of the transaction in the account's currency
//! - local_currency: the currency of the transaction
//! - local_amount: the amount of the transaction in the local currency
//! - category: the category of the transaction
//!

mod pots;
mod savings;

use std::fs::File;
use std::io::Write;

use chrono::NaiveDate;
use colored::Colorize;
use dialoguer::Confirm;
use serde::Deserialize;

use crate::beancount::directive::Directive;
use crate::beancount::transaction::{Postings, Transaction as BeancountTransaction};
use crate::error::AppError as Error;

use pots::process_pot;
use savings::process_savings;

pub async fn process(do_regenerate: bool) -> Result<(), Error> {
    println!("Processing CSV");

    if do_regenerate {
        if !confirm_reset()? {
            return Err(Error::AbortError);
        }

        process_savings()?;
    }

    let pots = vec!["essential-fixed", "essential-variable", "discretionary"];
    for pot in pots {
        process_pot(pot)?;
    }

    Ok(())
}

fn confirm_reset() -> Result<bool, Error> {
    println!("Resetting savings");
    println!(
        "{} {}",
        "WARNING".red(),
        "This destroys any manual data in Savings and cannot be undone".bold()
    );
    let confirmation = Confirm::new()
        .with_prompt("Do you want to continue?")
        .interact()
        .map_err(|_| Error::AbortError)?;

    Ok(confirmation)
}

#[derive(Debug, Deserialize, Clone)]
struct Record {
    date: NaiveDate,
    description: String,
    amount: f64,
    local_currency: Option<String>,
    local_amount: Option<f64>,
    category: Option<String>,
}

fn prepare_transaction(postings: &Postings, tx: &Record) -> BeancountTransaction {
    // let local amou

    // let local_amount = format!("{}", tx.local_amount.clone().map(|a| a.to_string()));

    let comment = tx.local_amount.clone().map(|a| a.to_string());
    let date = tx.date.clone();
    let notes = tx.description.clone();

    BeancountTransaction {
        comment,
        date,
        notes,
        postings: postings.clone(),
    }
}

fn write_directives(file: &mut File, directives: Vec<Directive>) -> Result<(), Error> {
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}
