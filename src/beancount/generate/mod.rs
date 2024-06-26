//! Processes inputs and generates a set of beancount accounts.
//!

pub(crate) mod classifier;
pub(crate) mod google_sheet_directives;
pub(crate) mod open_directives;

use std::fs;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

use classifier::{classify_transaction, Classification};
use config::Case;
use convert_case::Casing;
use rusty_money::{iso, Money};

use crate::beancount::generate::google_sheet_directives::google_sheet_directives;
use crate::beancount::google::transactions::Transaction as GoogleTransaction;
use crate::error::AppError as Error;

use super::google::GoogleSheetAccount;
use super::{
    account::{Account as BeancountAccount, AccountType},
    directive::Directive,
    transaction::{Posting, Postings, Transaction as BeancountTransaction},
    Beancount,
};

use open_directives::open_directives;

impl Beancount {
    /// Process the input and produce a set of Beancount accounts
    pub async fn generate(&self) -> Result<(), Error> {
        let option_directives = option_directives();

        let include_directives = include_directives(self.data_file_paths.include_dir.clone())?;

        let open_directives = open_directives(self.user_settings.clone()).await?;

        let transaction_directives = match &self.user_settings.googlesheet_accounts {
            Some(a) => google_sheet_directives(self, a).await?,
            None => vec![],
        };

        let mut file = File::create(self.data_file_paths.main_file.clone())?;
        write_directives(&mut file, option_directives)?;
        write_directives(&mut file, include_directives)?;
        write_directives(&mut file, open_directives)?;
        write_directives(&mut file, transaction_directives)?;

        Ok(())
    }
}

fn option_directives() -> Vec<Directive> {
    vec![
        Directive::Option("title".to_string(), "Monzo Accounts".to_string()),
        Directive::Option("operating_currency".to_string(), "GBP".to_string()),
    ]
}

fn include_directives(include_dir: PathBuf) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = vec![];
    let mut beanfiles = Vec::new();

    let files = fs::read_dir(include_dir)?;

    // Iterate over the directory entries
    for file in files {
        let entry = file?;
        let path = entry.path();

        // Check if the path is a file and has a .csv extension
        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("beancount")
        {
            beanfiles.push(path);
        }
    }

    for beanfile in beanfiles {
        let subpath = extract_last_two_components(&beanfile)?;
        let include_path = &subpath.to_string_lossy().to_string()[1..];
        directives.push(Directive::Include(include_path.to_string()));
    }

    Ok(directives)
}

fn extract_last_two_components(path: &Path) -> Result<PathBuf, Error> {
    let components: Vec<_> = path.components().collect();

    if components.len() >= 2 {
        let last_two = components[components.len() - 2..components.len()].iter();
        let mut subpath = PathBuf::new();
        subpath.push("/");
        for component in last_two {
            subpath.push(component.as_os_str());
        }
        Ok(subpath)
    } else {
        Err(Error::InvalidFileName(
            path.to_path_buf().to_string_lossy().to_string(),
        ))
    }
}

fn write_directives(file: &mut File, directives: Vec<Directive>) -> Result<(), Error> {
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}

fn prepare_to_posting(
    asset_accounts: &Vec<BeancountAccount>,
    income_accounts: &Vec<BeancountAccount>,
    account: &GoogleSheetAccount,
    tx: &GoogleTransaction,
) -> Result<Posting, Error> {
    let mut account = BeancountAccount {
        account_type: AccountType::Expenses,
        country: account.country.clone(),
        institution: account.institution.clone(),
        account: account.name.clone().to_case(Case::Pascal),
        sub_account: Some(tx.category.clone().to_case(Case::Pascal)),
        transaction_id: None,
    };
    let mut amount = -tx.amount as f64;

    #[allow(clippy::assigning_clones)] // TODO: Remove this
    if let Some(classification) = classify_transaction(asset_accounts, income_accounts, tx)? {
        match classification {
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
        }
    }

    // if tx.id == "tx_0000AhhIR9JeIvqoOGZt35".to_string() {
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

fn prepare_from_posting(
    asset_accounts: &Vec<BeancountAccount>,
    income_accounts: &Vec<BeancountAccount>,
    account: &GoogleSheetAccount,
    tx: &GoogleTransaction,
) -> Result<Posting, Error> {
    let mut amount = tx.amount as f64;

    let mut account = BeancountAccount {
        account_type: AccountType::Assets,
        country: account.country.clone(),
        institution: account.institution.clone(),
        account: account.name.clone().to_case(Case::Pascal),
        sub_account: None,
        transaction_id: Some(tx.id.clone()),
    };

    #[allow(clippy::assigning_clones)] // TODO: Remove this
    if let Some(classification) = classify_transaction(asset_accounts, income_accounts, tx)? {
        match classification {
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
        }
    }

    // if tx.id == "tx_0000AhhIR9JeIvqoOGZt35".to_string() {
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
    let date = tx.date;
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
    tx.name.clone()
}

fn prepare_amount(tx: &GoogleTransaction) -> String {
    if tx.currency == tx.local_currency {
        String::new()
    } else if let Some(iso_code) = iso::find(&tx.local_currency) {
        format!("{}", Money::from_minor(tx.local_amount, iso_code))
    } else {
        format!("{} {}", tx.local_amount, tx.local_currency)
    }
}
