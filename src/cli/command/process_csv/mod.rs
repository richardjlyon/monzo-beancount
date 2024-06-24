//! Process CSV command
//!

mod savings;

use std::fs::File;
use std::io::Write;

use chrono::NaiveDate;
use serde::Deserialize;

use crate::beancount::directive::Directive;
use crate::beancount::transaction::{Postings, Transaction as BeancountTransaction};
use crate::error::AppError as Error;

use savings::process_savings;

pub async fn process() -> Result<(), Error> {
    println!("Processing CSV");
    process_savings()?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Record {
    date: NaiveDate,
    description: String,
    amount: f64,
    local_currency: Option<String>,
    local_amount: Option<f64>,
    category: Option<String>,
}

fn prepare_transaction(postings: &Postings, tx: &Record) -> BeancountTransaction {
    let comment = None;
    let date = tx.date.clone();
    let notes = "Interest".to_string();

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
