//! Convert CSV files to beancount files and place in the `include` directory.
//!
//! Process a set of CSV files and generate Beancount directives. The CSV files are expected to be
//! in a specific format, with the following columns:
//!
//! ```text
//! date,description,amount,local_currency,local_amount,category
//! 2024-04-14,PATH TAPP PAYGO CP NEW JERSEY USA,-0.8,USD,-1.0,Transport
//! ```
//! - **date**: the date of the transaction
//! - **description**: a description of the transaction
//! - **amount**: the amount of the transaction in the account's currency
//! - **local_currency**: the currency of the transaction
//! - **local_amount**: the amount of the transaction in the local currency
//! - **category**: the category of the transaction
//!

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use config::Case;
use convert_case::Casing;
use csv::Reader;
use serde::Deserialize;

use crate::beancount::datafile_paths::DataFilePaths;
use crate::beancount::transaction::Postings;

use crate::{
    beancount::{
        account::{Account, AccountType},
        directive::Directive,
        transaction::{Posting, Transaction as BeancountTransaction},
        Beancount,
    },
    error::AppError as Error,
};

#[derive(Debug, Deserialize, Clone)]
/// Represents a single transaction record.
struct Record {
    date: NaiveDate,
    description: String,
    amount: f64,
    local_currency: Option<String>,
    local_amount: Option<f64>,
    category: Option<String>,
}

/// Imports the CSV files from the `import` directory and generates Beancount files.
pub async fn import(beancount: &Beancount) -> Result<(), Error> {
    let csv_files = get_csv_files(&beancount.data_file_paths.import_dir)?;

    for csv_file in csv_files {
        let directives = process_csv_file(&csv_file)?;
        let mut beancount_file = beanacount_file(&csv_file, &beancount.data_file_paths)?;
        write_directives(&mut beancount_file, directives)?;
    }

    Ok(())
}

// get the .CSV files in `dir`
fn get_csv_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut csv_files = Vec::new();

    // Read the directory contents
    let entries = fs::read_dir(dir)?;

    // Iterate over the directory entries
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Check if the path is a file and has a .csv extension
        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("csv") {
            csv_files.push(path);
        }
    }

    Ok(csv_files)
}

/// Read a CSV file of transactions and create a Beancount file
pub(crate) fn process_csv_file(csv_file: &PathBuf) -> Result<Vec<Directive>, Error> {
    let records = get_sorted_records(csv_file)?;
    let account_name = account_name_from_csv_file(csv_file);
    let mut directives: Vec<Directive> = vec![];

    directives.push(Directive::Comment(account_name.to_string()));
    directives.push(Directive::Comment("Transactions".to_string()));
    directives.extend(generate_directives(records.clone(), &account_name)?);
    directives.push(close_account(&records, &account_name));

    Ok(directives)
}

// e.g. essential-variable-pot.csv -> EssentialVariablePot
fn account_name_from_csv_file(csv_file: &Path) -> String {
    let account_name = csv_file
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_case(Case::Pascal);

    account_name
}

// e.g./a/b/include/essential-variable-pot.csv -> /a/b/accounts/essential-variable-pot.beanfile
fn beanacount_file(csv_file: &Path, file_paths: &DataFilePaths) -> Result<File, Error> {
    // Create a new path by iterating over the components of the original path
    let csv_file_name = csv_file
        .file_name()
        .and_then(|file_name| file_name.to_str()) // Convert OsStr to &str
        .map(|file_name_str| file_name_str.to_string())
        .ok_or_else(|| Error::InvalidFileName(csv_file.display().to_string()))?;

    // Change the file name extension
    let beancount_file_name = csv_file_name.replace(".csv", ".beancount");
    let beancount_file_path = file_paths.include_dir.join(beancount_file_name);
    let beancount_file = File::create(beancount_file_path)?;

    Ok(beancount_file)
}

// deserialise the records from the CSV file
fn get_sorted_records(csv_file_path: &PathBuf) -> Result<Vec<Record>, Error> {
    let mut reader = Reader::from_path(csv_file_path).expect("Failed to open CSV file");
    let mut records: Vec<Record> = reader
        .deserialize()
        .filter_map(|result| result.ok())
        .collect();

    records.sort_by_key(|record| record.date);

    Ok(records)
}

fn generate_directives(records: Vec<Record>, pot_name: &str) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = vec![];

    for record in &records {
        let to_posting = prepare_to_posting(record, pot_name)?;
        let from_posting = prepare_from_posting(record, pot_name)?;

        let postings = Postings {
            to: to_posting,
            from: from_posting,
        };

        let transaction = prepare_transaction(&postings, record);

        directives.push(Directive::Transaction(Box::new(transaction)));
    }

    Ok(directives)
}

fn prepare_to_posting(record: &Record, pot_name: &str) -> Result<Posting, Error> {
    let account = if is_income(&record.category.clone().unwrap_or("".to_string()))
        || is_transfer(&record.description)
    {
        Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: pot_name.to_string(),
            sub_account: None,
            transaction_id: None,
        }
    } else {
        Account {
            account_type: AccountType::Expenses,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: record.category.clone(),
            transaction_id: None,
        }
    };

    let amount = if is_transfer(&record.description)
        || is_income(&record.category.clone().unwrap_or("".to_string()))
    {
        record.amount * 100.0
    } else {
        -record.amount * 100.0
    };

    let currency = "GBP".to_string();
    let description = Some(record.description.clone());

    Ok(Posting {
        account,
        amount,
        currency,
        description,
    })
}

fn prepare_from_posting(record: &Record, pot_name: &str) -> Result<Posting, Error> {
    // let category = record.category.clone().unwrap_or("".to_string());
    let account = if is_income(&record.category.clone().unwrap_or("".to_string())) {
        Account {
            account_type: AccountType::Income,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: pot_name.to_string(),
            sub_account: None,
            transaction_id: None,
        }
    } else if is_transfer(&record.description) {
        Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
            transaction_id: None,
        }
    } else {
        Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: pot_name.to_string(),
            sub_account: None,
            transaction_id: None,
        }
    };

    let amount = if is_transfer(&record.description)
        || is_income(&record.category.clone().unwrap_or("".to_string()))
    {
        record.amount * -100.0
    } else {
        record.amount * 100.0
    };

    let currency = "GBP".to_string();
    let description = Some(record.description.clone());

    Ok(Posting {
        account,
        amount,
        currency,
        description,
    })
}

fn is_transfer(description: &str) -> bool {
    description.starts_with("Withdrawal") || description.starts_with("Deposit")
}

fn is_income(category: &str) -> bool {
    category == "Income"
}

fn close_account(records: &[Record], pot_name: &str) -> Directive {
    let last_record = records.last().unwrap();

    // Close the account
    let account = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: pot_name.to_string(),
        sub_account: None,
        transaction_id: None,
    };

    Directive::Close(
        last_record.date,
        account,
        Some(format!("Close {}", pot_name.to_case(Case::Title))),
    )
}

fn prepare_transaction(postings: &Postings, tx: &Record) -> BeancountTransaction {
    let date = tx.date;
    let mut notes = tx.description.clone();

    // Convert local_amount to string or initialize to empty string if None.
    let mut comment = tx.local_amount.map_or(String::new(), |a| a.to_string());

    // Append local_amount and local_currency to notes if both are Some.
    if let (Some(local_amount), Some(local_currency)) =
        (tx.local_amount, tx.local_currency.as_ref())
    {
        notes += &format!(" {} {}", local_amount, local_currency);
    }

    // Remove everything from "Amount" onwards in comment if found.
    if let Some(index) = comment.find("Amount") {
        comment.truncate(index);
    }

    BeancountTransaction {
        comment: Some(comment),
        date,
        notes,
        postings: postings.clone(),
    }
}

fn write_directives(beancount_file: &mut File, directives: Vec<Directive>) -> Result<(), Error> {
    for d in directives {
        beancount_file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}
