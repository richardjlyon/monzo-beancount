//! Functionality for processing inputs and producing a set of beancount accounts.
//!

pub(crate) mod csv_directives;
pub(crate) mod google_sheet_directives;
pub(crate) mod manual_directives;
pub(crate) mod open_directives;

use std::collections::HashSet;
use std::{fs::File, io::Write};

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

use csv_directives::csv_directives;
use google_sheet_directives::google_sheet_directives;
use manual_directives::manual_directives;
use open_directives::open_directives;

impl Beancount {
    /// Process the input and produce a set of Beancount accounts
    pub async fn generate(&self) -> Result<(), Error> {
        let option_directives = option_directives();

        let open_directives = open_directives().await?;

        let mut transaction_directives = google_sheet_directives().await?;
        transaction_directives.extend(csv_directives()?);
        transaction_directives.extend(manual_directives()?);

        // TODO: sort transaction_directives by date

        // write beancount files to disk
        let file_path = self.file_paths.root_dir.join("main.beancount");
        let mut file = File::create(file_path)?;

        write_directives(&mut file, option_directives)?;
        write_directives(&mut file, open_directives)?;
        write_directives(&mut file, transaction_directives)?;

        Ok(())
    }
}

fn option_directives() -> Vec<Directive> {
    vec![
        Directive::Option(
            "title".to_string(),
            "My Excellent Beancount File".to_string(),
        ),
        Directive::Option("operating_currency".to_string(), "GBP".to_string()),
    ]
}

fn write_directives(file: &mut File, directives: Vec<Directive>) -> Result<(), Error> {
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }
    // let _ = file.write_all("\n".as_bytes())?;

    Ok(())
}

fn prepare_to_posting(account: &GoogleAccount, tx: &GoogleTransaction) -> Result<Posting, Error> {
    let mut amount = -tx.amount as f64;

    let mut account = BeancountAccount {
        account_type: AccountType::Expenses,
        country: account.country.clone(),
        institution: account.institution.clone(),
        account: account.name.clone().to_case(Case::Pascal),
        sub_account: Some(tx.category.clone().to_case(Case::Pascal)),
    };

    match tx.category.as_str() {
        "Transfers" => {
            // pot tranfer
            if tx.payment_type == "Pot transfer" {
                account.account_type = AccountType::Assets;
                account.sub_account = Some(tx.name.clone());

            // equity opening balance
            } else if tx
                .description
                .clone()
                .unwrap_or_default()
                .starts_with("Monzo-")
            {
                account.account_type = AccountType::Assets;
                account.sub_account = None;
                amount = tx.amount as f64;

            // named asset
            } else if let Some(matching_asset) = matching_asset(&tx.name) {
                account.account_type = AccountType::Assets;
                account.institution = matching_asset.institution;
                account.account = tx.name.clone();
                account.sub_account = None;

            // other asset
            } else {
                account.account_type = AccountType::Assets;
                account.sub_account = None;
            }
        }
        "Savings" => {
            account.account_type = AccountType::Assets;
            account.sub_account = Some("Savings".to_string());
        }
        "Income" => {
            account.account_type = AccountType::Assets;
            account.sub_account = None;
            amount = tx.amount as f64;
        }
        _ => {}
    }

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
    };

    match tx.category.as_str() {
        "Transfers" => {
            // pot transfer
            if tx.payment_type == "Pot transfer" {
                account.account_type = AccountType::Assets;
                account.sub_account = None;

            // equity opening balance
            } else if let Some(description) = &tx.description {
                if description.starts_with("Monzo-") {
                    account.account_type = AccountType::Equity;
                    account.account = "OpeningBalances".to_string();
                    amount = -tx.amount as f64;
                }
            } else {
                account.account_type = AccountType::Expenses;
                account.sub_account = None;
            }
        }
        "Savings" => {
            account.account_type = AccountType::Assets;
            account.sub_account = None;
        }
        "Income" => {
            if let Some(matching_income) = matching_income(&tx.name) {
                account.account_type = AccountType::Income;
                account.institution = matching_income.institution;
                account.account = tx.name.clone();
                account.sub_account = None;
                amount = -tx.amount as f64;
            } else {
                account.account_type = AccountType::Income;
                account.sub_account = None;
                amount = -tx.amount as f64;
            }
        }
        _ => {}
    }

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
    let merchant_name = tx.name.clone();

    format!("{}", merchant_name)
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

fn get_filtered_assets() -> Result<Vec<BeancountAccount>, Error> {
    let bc = Beancount::from_config()?;
    let assets = bc.assets.unwrap();
    let unwanted_accounts = vec!["Business", "Personal"];

    let unique_accounts: HashSet<BeancountAccount> = assets
        .into_iter()
        .filter(|a| !unwanted_accounts.contains(&a.account.as_str()))
        .collect();

    let unique_accounts_vec: Vec<BeancountAccount> = unique_accounts.into_iter().collect();

    Ok(unique_accounts_vec)
}

fn matching_asset(account_to_find: &str) -> Option<BeancountAccount> {
    let filtered_assets = get_filtered_assets().unwrap();
    let matching_assets: Vec<BeancountAccount> = filtered_assets
        .iter()
        .filter(|&asset| asset.account == account_to_find)
        .cloned() // Use `cloned` to get ownership of the filtered assets
        .collect();
    match matching_assets.len() {
        1 => Some(matching_assets[0].clone()),
        _ => None,
    }
}

fn get_filtered_income() -> Result<Vec<BeancountAccount>, Error> {
    let bc = Beancount::from_config()?;
    let income = bc.income.unwrap();
    let unwanted_accounts = vec!["Business", "Personal"];

    let unique_accounts: HashSet<BeancountAccount> = income
        .into_iter()
        .filter(|a| !unwanted_accounts.contains(&a.account.as_str()))
        .collect();

    let unique_accounts_vec: Vec<BeancountAccount> = unique_accounts.into_iter().collect();

    Ok(unique_accounts_vec)
}

fn matching_income(account_to_find: &str) -> Option<BeancountAccount> {
    let filtered_income = get_filtered_income().unwrap();
    let matching_income: Vec<BeancountAccount> = filtered_income
        .iter()
        .filter(|&income| income.account == account_to_find)
        .cloned() // Use `cloned` to get ownership of the filtered assets
        .collect();
    match matching_income.len() {
        1 => Some(matching_income[0].clone()),
        _ => None,
    }
}
