use std::fs::File;

use config::Case;
use convert_case::Casing;
use csv::Reader;

use crate::{
    beancount::{
        account::{Account, AccountType},
        directive::Directive,
        transaction::{Posting, Postings},
        Beancount,
    },
    error::AppError as Error,
};

use super::{prepare_transaction, write_directives, Record};

/// Read a CSV file of Monzo savings pot transactions and create a Beancount file
/// from the interest payments
pub(crate) fn process_pot(pot_name: &str) -> Result<(), Error> {
    let records = get_sorted_records(pot_name)?;
    let mut beancount_file = get_beancount_file_path(pot_name);
    let pot_name = format!("{}Pot", pot_name.to_case(Case::Pascal));

    let mut directives: Vec<Directive> = vec![];
    directives.push(Directive::Comment(pot_name.to_string()));
    directives.extend(generate_directives(records.clone(), &pot_name)?);
    directives.push(close_account(&records, &pot_name));

    write_directives(&mut beancount_file, directives)?;

    Ok(())
}

// deserialise the records from the CSV file
fn get_sorted_records(pot_name: &str) -> Result<Vec<Record>, Error> {
    let csv_file_folder =
        "/Users/richardlyon/SynologyDrive/[01] Areas/[04] Money/[02] banks/Monzo/transactions/";
    let csv_file_path = format!("{}{}.csv", csv_file_folder, pot_name);
    let mut reader = Reader::from_path(csv_file_path).expect("Failed to open CSV file");
    let mut records: Vec<Record> = reader
        .deserialize()
        .filter_map(|result| result.ok())
        .collect();

    records.sort_by_key(|record| record.date);

    Ok(records)
}

fn get_beancount_file_path(pot_name: &str) -> File {
    let bean = Beancount::from_config().unwrap();
    let beancount_file_path = bean
        .file_paths
        .include_dir
        .join(format!("{}.beancount", pot_name));
    let beancount_file = File::create(beancount_file_path).expect("Failed to create file");

    beancount_file
}

fn generate_directives(records: Vec<Record>, pot_name: &str) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = vec![];

    for record in &records {
        let to_posting = prepare_to_posting(&record, pot_name)?;
        let from_posting = prepare_from_posting(&record)?;

        let postings = Postings {
            to: to_posting,
            from: from_posting,
        };

        let transaction = prepare_transaction(&postings, &record);

        directives.push(Directive::Transaction(transaction));
    }

    Ok(directives)
}

fn close_account(records: &Vec<Record>, pot_name: &str) -> Directive {
    let last_record = records.last().unwrap();

    // Close the account
    let account = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Personal".to_string(),
        sub_account: Some(pot_name.to_string()),
        transaction_id: None,
    };

    Directive::Close(
        last_record.date,
        account,
        Some(format!(
            "Close {}",
            pot_name.to_case(Case::Title).to_string()
        )),
    )
}

fn prepare_to_posting(record: &Record, pot_name: &str) -> Result<Posting, Error> {
    let account = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Personal".to_string(),
        sub_account: Some(pot_name.to_string()),
        transaction_id: None,
    };

    let amount = record.amount * 100.0;

    let currency = "GBP".to_string();
    let description = Some(record.description.clone());

    Ok(Posting {
        account,
        amount,
        currency,
        description,
    })
}

fn prepare_from_posting(record: &Record) -> Result<Posting, Error> {
    let account = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Personal".to_string(),
        sub_account: None,
        transaction_id: None,
    };

    let amount = -record.amount * 100.0;

    let currency = "GBP".to_string();
    let description = Some(record.description.clone());

    Ok(Posting {
        account,
        amount,
        currency,
        description,
    })
}
