//! Update transactions command

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use config::Case;
use convert_case::Casing;
use rusty_money::{iso, Money};

use crate::beancount::directive::Directive;
use crate::beancount::transaction::{Posting, Postings, Transaction as BeancountTransaction};
use crate::beancount::{Account as BeancountAccount, AccountType, Beancount};
use crate::error::AppError as Error;
use crate::google;
use crate::google::config::{load_sheets, GoogleAccount};
use crate::google::transactions::Transaction as GoogleTransaction;

pub async fn update() -> Result<(), Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let bean = Beancount::from_config()?;

    // -- Initialise the file system -----------------------------------------------------

    match bean.initialise_filesystem()? {
        Some(message) => println!("{}", message),
        None => {}
    };

    // -- Open Equity Accounts -----------------------------------------------------

    directives.push(Directive::Comment("equity accounts".to_string()));
    directives.extend(open_config_equity_accounts()?);

    // -- Open Asset Accounts --------------------------------------------------------------

    directives.push(Directive::Comment("asset accounts".to_string()));
    directives.extend(open_config_assets()?);

    // Open Liability Accounts ---------------------------------------------------------

    directives.push(Directive::Comment("liability accounts".to_string()));
    directives.extend(open_config_liabilities().await?);

    // -- Open Income Accounts ---------------------------------------------------------

    directives.push(Directive::Comment("income accounts".to_string()));
    directives.extend(open_config_income()?);

    // -- Open Expense Accounts  ---------------------------------------------------------

    directives.push(Directive::Comment("Expense accounts".to_string()));
    directives.extend(open_expenses().await?);

    // -- Post Sheet Transactions---------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));
    directives.extend(post_transactions().await?);

    // push transactions from google sheets
    // push transactions from Pot CSV files

    // for tx in transactions {
    //     println!("{:#?}", tx);
    // }

    // -- Write directives to file -----------------------------------------------------

    let file_path = bean.settings.root_dir.join("report.beancount");
    let mut file = File::create(file_path)?;
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}

fn open_config_equity_accounts() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(equity_accounts) = bc.settings.equity {
        for equity in equity_accounts {
            directives.push(Directive::OpenEquity(bc.settings.start_date, equity, None));
        }
    }

    Ok(directives)
}

fn open_config_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    match bc.settings.assets {
        Some(asset_accounts) => {
            for asset_account in asset_accounts {
                directives.push(Directive::OpenAccount(open_date, asset_account, None));
            }
        }
        None => (),
    }

    Ok(directives)
}

fn open_config_income() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    match bc.settings.income {
        Some(income_account) => {
            for income_account in income_account {
                directives.push(Directive::OpenAccount(open_date, income_account, None));
            }
        }
        None => (),
    }

    Ok(directives)
}

// Open a liability account for each config file entity
async fn open_config_liabilities() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.settings.liabilities.is_none() {
        return Ok(directives);
    }

    // open configured liabilities
    for account in bc.settings.liabilities.unwrap() {
        directives.push(Directive::OpenAccount(open_date, account, None));
    }

    Ok(directives)
}

// Open expense accounts for each Category in the Google Sheets
async fn open_expenses() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.settings.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    let accounts = load_sheets()?;

    for account in accounts {
        let sheet = google::GoogleSheet::new(account.clone()).await?;
        let expense_accounts = sheet.expense_accounts().await?;
        for expense_account in expense_accounts {
            let beanaccount = BeancountAccount {
                account_type: AccountType::Expenses,
                country: account.country.clone(),
                institution: account.institution.clone(),
                account: account.name.clone(),
                sub_account: Some(expense_account),
            };
            directives.push(Directive::OpenAccount(open_date, beanaccount, None));
        }
    }

    Ok(directives)
}

async fn post_transactions() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let accounts = load_sheets()?;

    for account in accounts {
        let sheet = google::GoogleSheet::new(account.clone()).await?;

        if let Some(transactions) = sheet.transactions() {
            for tx in transactions {
                let to_posting = prepare_to_posting(&account, tx)?;
                let from_posting = prepare_from_posting(&account, tx)?;

                let postings = Postings {
                    to: to_posting,
                    from: from_posting,
                };

                let transaction = prepare_transaction(&postings, tx);

                directives.push(Directive::Transaction(transaction));
            }
        }
    }

    Ok(directives)
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
                    account.sub_account = Some("OpeningBalances".to_string());
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
    let assets = bc.settings.assets.unwrap();
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
    let income = bc.settings.income.unwrap();
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
