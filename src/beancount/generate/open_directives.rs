//! Generate open directives for the beancount file.

use crate::beancount::account::{Account, AccountType};
use crate::beancount::directive::Directive;

use crate::beancount::google::GoogleSheet;
use crate::beancount::user_settings::UserSettings;
use crate::error::AppError as Error;

pub(crate) async fn open_directives(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    // -- Open Equity Accounts -----------------------------------------------------

    directives.push(Directive::Comment("equity accounts".to_string()));
    directives.extend(open_equity_account(user_settings.clone())?);

    // -- Open Asset Accounts --------------------------------------------------------------

    directives.push(Directive::Comment("asset accounts".to_string()));
    directives.extend(open_config_assets(user_settings.clone())?);

    // Open Liability Accounts ---------------------------------------------------------

    directives.push(Directive::Comment("liability accounts".to_string()));
    directives.extend(open_config_liabilities(user_settings.clone()).await?);

    // -- Open Income Accounts ---------------------------------------------------------

    directives.push(Directive::Comment("income accounts".to_string()));
    directives.extend(open_config_income(user_settings.clone())?);

    // -- Open Expense Accounts  ---------------------------------------------------------

    directives.push(Directive::Comment("Expense accounts".to_string()));
    directives.extend(open_expenses(user_settings.clone()).await?);
    directives.extend(open_config_expenses(user_settings.clone()).await?);

    Ok(directives)
}

fn open_equity_account(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let equity_account = Account {
        account_type: AccountType::Equity,
        country: "GBP".to_string(),
        institution: String::new(),
        account: "Opening Balances".to_string(),
        sub_account: None,
        transaction_id: None,
    };
    directives.push(Directive::Open(
        user_settings.start_date,
        equity_account.clone(),
        None,
    ));

    Ok(directives)
}

fn open_config_assets(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(asset_accounts) = user_settings.assets {
        for asset_account in asset_accounts {
            directives.push(Directive::Open(
                user_settings.start_date,
                asset_account,
                None,
            ));
        }
    }

    Ok(directives)
}

fn open_config_income(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(income_account) = user_settings.income {
        for income_account in income_account {
            directives.push(Directive::Open(
                user_settings.start_date,
                income_account,
                None,
            ));
        }
    }

    Ok(directives)
}

// Open a liability account for each config file entity
async fn open_config_liabilities(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    if user_settings.liabilities.is_none() {
        return Ok(directives);
    }

    // open configured liabilities
    for account in user_settings.liabilities.unwrap() {
        directives.push(Directive::Open(user_settings.start_date, account, None));
    }

    Ok(directives)
}

// Open expense accounts for each Category in the Google Sheets
async fn open_expenses(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let googlesheet_accounts = match user_settings.googlesheet_accounts {
        Some(acc) => acc,
        None => return Ok(directives),
    };

    for googlesheet_account in googlesheet_accounts {
        let google_sheet = GoogleSheet::new(googlesheet_account.clone()).await?;
        let expense_accounts = google_sheet.expense_accounts().await?;
        for expense_account in expense_accounts {
            let beanaccount = Account {
                account_type: AccountType::Expenses,
                country: googlesheet_account.country.clone(),
                institution: googlesheet_account.institution.clone(),
                account: googlesheet_account.name.clone(),
                sub_account: Some(expense_account),
                transaction_id: None,
            };
            directives.push(Directive::Open(user_settings.start_date, beanaccount, None));
        }
    }

    Ok(directives)
}

async fn open_config_expenses(user_settings: UserSettings) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(expense_accounts) = user_settings.expenses {
        for expense_account in expense_accounts {
            directives.push(Directive::Open(
                user_settings.start_date,
                expense_account,
                None,
            ));
        }
    }

    Ok(directives)
}
