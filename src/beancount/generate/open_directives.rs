//! Generate open directives for the beancount file.

use crate::beancount::account::{Account, AccountType};
use crate::beancount::directive::Directive;
use crate::beancount::google::config::load_sheets;
use crate::beancount::google::GoogleSheet;
use crate::beancount::Beancount;
use crate::error::AppError as Error;

pub(crate) async fn open_directives() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    println!("creating open directives");

    // -- Open Equity Accounts -----------------------------------------------------

    directives.push(Directive::Comment("equity accounts".to_string()));
    directives.extend(open_equity_account()?);

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
    directives.extend(open_config_expenses().await?);

    Ok(directives)
}

fn open_equity_account() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let mut directives: Vec<Directive> = Vec::new();

    let equity_account = Account {
        account_type: AccountType::Equity,
        country: "GBP".to_string(),
        institution: String::new(),
        account: "Opening Balances".to_string(),
        sub_account: None,
        transaction_id: None,
    };

    directives.push(Directive::Open(bc.start_date, equity_account.clone(), None));

    Ok(directives)
}

fn open_config_assets() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(asset_accounts) = bc.assets {
        for asset_account in asset_accounts {
            directives.push(Directive::Open(open_date, asset_account, None));
        }
    }

    Ok(directives)
}

fn open_config_income() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(income_account) = bc.income {
        for income_account in income_account {
            directives.push(Directive::Open(open_date, income_account, None));
        }
    }

    Ok(directives)
}

// Open a liability account for each config file entity
async fn open_config_liabilities() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if bc.liabilities.is_none() {
        return Ok(directives);
    }

    // open configured liabilities
    for account in bc.liabilities.unwrap() {
        directives.push(Directive::Open(open_date, account, None));
    }

    Ok(directives)
}

// Open expense accounts for each Category in the Google Sheets
async fn open_expenses() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    let accounts = load_sheets()?;

    for account in accounts {
        let sheet = GoogleSheet::new(account.clone()).await?;
        let expense_accounts = sheet.expense_accounts().await?;
        for expense_account in expense_accounts {
            let beanaccount = Account {
                account_type: AccountType::Expenses,
                country: account.country.clone(),
                institution: account.institution.clone(),
                account: account.name.clone(),
                sub_account: Some(expense_account),
                transaction_id: None,
            };
            directives.push(Directive::Open(open_date, beanaccount, None));
        }
    }

    Ok(directives)
}

async fn open_config_expenses() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let open_date = bc.start_date;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(expense_accounts) = bc.expenses {
        for expense_account in expense_accounts {
            directives.push(Directive::Open(open_date, expense_account, None));
        }
    }

    Ok(directives)
}
