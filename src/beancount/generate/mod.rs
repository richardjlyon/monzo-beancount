//! Functionality for processing inputs and producing a set of beancount accounts.
//!

pub(crate) mod classifier;
pub(crate) mod google_sheet_directives;
pub(crate) mod open_directives;

use std::{fs::File, io::Write};

use classifier::{classify_transaction, Classification};
use config::Case;
use convert_case::Casing;
use rusty_money::{iso, Money};

use crate::google::transactions::Transaction as GoogleTransaction;
use crate::{error::AppError as Error, google::config::GoogleAccount};

use super::{
    account::{Account as BeancountAccount, AccountType},
    directive::Directive,
    transaction::{Posting, Postings, Transaction as BeancountTransaction},
    Beancount,
};

use google_sheet_directives::google_sheet_directives;
use open_directives::open_directives;

impl Beancount {
    /// Process the input and produce a set of Beancount accounts
    pub async fn generate(&self) -> Result<(), Error> {
        let option_directives = option_directives();

        let open_directives = open_directives().await?;

        let transaction_directives = google_sheet_directives().await?;

        // TODO: sort transaction_directives by date

        // write beancount files to disk
        let mut file = File::create(self.file_paths.main_file.clone())?;

        write_directives(&mut file, option_directives)?;
        write_directives(&mut file, open_directives)?;
        write_directives(&mut file, transaction_directives)?;

        Ok(())
    }
}

fn option_directives() -> Vec<Directive> {
    vec![
        Directive::Option("title".to_string(), "Monzo Accounts".to_string()),
        Directive::Option("operating_currency".to_string(), "GBP".to_string()),
        Directive::Include("include/savings.beancount".to_string()),
        Directive::Include("include/essential-fixed.beancount".to_string()),
        Directive::Include("include/essential-variable.beancount".to_string()),
        Directive::Include("include/discretionary.beancount".to_string()),
    ]
}

fn write_directives(file: &mut File, directives: Vec<Directive>) -> Result<(), Error> {
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}

fn prepare_to_posting(account: &GoogleAccount, tx: &GoogleTransaction) -> Result<Posting, Error> {
    let mut account = BeancountAccount {
        account_type: AccountType::Expenses,
        country: account.country.clone(),
        institution: account.institution.clone(),
        account: account.name.clone().to_case(Case::Pascal),
        sub_account: Some(tx.category.clone().to_case(Case::Pascal)),
        transaction_id: None,
    };
    let mut amount = -tx.amount as f64;

    match classify_transaction(tx)? {
        Some(classification) => match classification {
            Classification::IncomeGeneral => {
                // OK
                account.account_type = AccountType::Assets;
                account.sub_account = None;
                amount = tx.amount as f64;
            }
            Classification::IncomeAccount(_institution_account) => {
                account.account_type = AccountType::Assets;
                account.sub_account = None;
                amount = tx.amount as f64;
            }
            Classification::Savings => {
                account.account_type = AccountType::Assets;
                account.sub_account = Some("Savings".to_string());
            }
            Classification::TransferOpeningBalance => {
                account.account_type = AccountType::Assets;
                account.sub_account = None;
                amount = tx.amount as f64;
            }
            Classification::TransferPot => {
                account.account_type = AccountType::Assets;
                account.sub_account = Some(tx.name.clone());
            }
            Classification::TransferAsset(asset_account) => {
                // OK
                account.account_type = AccountType::Assets;
                account.institution = asset_account.institution;
                account.account = asset_account.account.clone();
                account.sub_account = None;
            }
        },
        None => {}
    }

    // if tx.id == "tx_0000Aew2N55dFgHod1VPco".to_string() {
    //     println!("TO:");
    //     println!("{:?}", classify_transaction(tx));
    //     println!("{:?}", tx);
    //     println!("{:#?}", account);
    // }

    Ok(Posting {
        account,
        amount,
        currency: tx.currency.to_string(),
        description: tx.description.clone(),
    })
}

fn prepare_from_posting(account: &GoogleAccount, tx: &GoogleTransaction) -> Result<Posting, Error> {
    let mut amount = tx.amount as f64;

    let mut account = BeancountAccount {
        account_type: AccountType::Assets,
        country: account.country.clone(),
        institution: account.institution.clone(),
        account: account.name.clone().to_case(Case::Pascal),
        sub_account: None,
        transaction_id: Some(tx.id.clone()),
    };

    match classify_transaction(tx)? {
        Some(classification) => match classification {
            Classification::IncomeGeneral => {
                account.account_type = AccountType::Income;
                amount = -tx.amount as f64;
            }
            Classification::IncomeAccount(income_account) => {
                account.account_type = AccountType::Income;
                account.institution = income_account.institution;
                account.account = tx.name.clone();
                amount = -tx.amount as f64;
            }
            Classification::Savings => {
                account.account_type = AccountType::Assets;
            }
            Classification::TransferOpeningBalance => {
                account.account_type = AccountType::Equity;
                account.account = "OpeningBalances".to_string();
                amount = -tx.amount as f64;
            }
            Classification::TransferPot => {
                account.account_type = AccountType::Assets;
            }
            Classification::TransferAsset(_asset_account) => {}
        },
        None => {}
    }

    // if tx.id == "tx_0000Aew2N55dFgHod1VPco".to_string() {
    //     println!("\nFROM:");
    //     println!("{:?}", classify_transaction(tx));
    //     println!("{:?}", tx);
    //     println!("{:#?}", account);
    // }

    Ok(Posting {
        account,
        amount,
        currency: tx.currency.to_string(),
        description: None,
    })
}

fn prepare_transaction(postings: &Postings, tx: &GoogleTransaction) -> BeancountTransaction {
    let comment = prepare_transaction_comment(tx);
    let date = tx.date.clone();
    let notes = prepare_transaction_notes(tx);

    BeancountTransaction {
        comment,
        date,
        notes,
        postings: postings.clone(),
    }
}

fn prepare_transaction_comment(tx: &GoogleTransaction) -> Option<String> {
    let amount = prepare_amount(tx);
    let notes = tx.notes.clone().unwrap_or_default();

    Some(format!("{notes} {amount}"))
}

fn prepare_transaction_notes(tx: &GoogleTransaction) -> String {
    // FIXME remove id after debugging
    let merchant_name = tx.name.clone();

    format!("{} - {}", merchant_name, tx.id)
}

fn prepare_amount(tx: &GoogleTransaction) -> String {
    if tx.currency == tx.local_currency {
        String::new()
    } else {
        if let Some(iso_code) = iso::find(&tx.local_currency) {
            format!("{}", Money::from_minor(tx.local_amount, iso_code))
        } else {
            format!("{} {}", tx.local_amount, tx.local_currency)
        }
    }
}
