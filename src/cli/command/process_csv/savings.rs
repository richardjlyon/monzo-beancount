use std::fs::File;

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
pub(crate) fn process_savings() -> Result<(), Error> {
    let csv_file_path = "/Users/richardlyon/SynologyDrive/[01] Areas/[04] Money/[02] banks/Monzo/transactions/savings.csv";
    let mut reader = Reader::from_path(csv_file_path).expect("Failed to open CSV file");

    let bean = Beancount::from_config().unwrap();
    let beancount_file_path = bean.file_paths.include_dir.join("savings.beancount");
    let mut beancount_file = File::create(beancount_file_path).expect("Failed to create file");

    let mut directives: Vec<Directive> = vec![];

    // Collect records where the description is "Interest"
    let mut records: Vec<Record> = reader
        .deserialize()
        .filter_map(|result| result.ok())
        .collect();

    records.sort_by_key(|record| record.date);

    // Create the directives
    directives.push(Directive::Comment("Savings Interest".to_string()));
    for record in records {
        let to_posting = prepare_to_posting(&record)?;
        let from_posting = prepare_from_posting(&record)?;

        let postings = Postings {
            to: to_posting,
            from: from_posting,
        };

        let transaction = prepare_transaction(&postings, &record);

        directives.push(Directive::Transaction(transaction));
    }

    // write beancount files to disk
    write_directives(&mut beancount_file, directives)?;

    Ok(())
}

fn prepare_to_posting(record: &Record) -> Result<Posting, Error> {
    let account = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Personal".to_string(),
        sub_account: Some("Savings".to_string()),
        transaction_id: None,
    };
    let amount = record.amount * 100.0;
    let currency = "GBP".to_string();
    let description = Some("Interest".to_string());

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
    let description = Some("Interest".to_string());

    Ok(Posting {
        account,
        amount,
        currency,
        description,
    })
}
