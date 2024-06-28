//! Process Google Sheet inputs and generate transaction directives.

use crate::beancount::google::{GoogleSheet, GoogleSheetAccount};
use crate::beancount::Beancount;
use crate::beancount::{directive::Directive, transaction::Postings};
use crate::error::AppError as Error;

use super::{prepare_from_posting, prepare_to_posting, prepare_transaction};

pub(crate) async fn google_sheet_directives(
    beancount: &Beancount,
    googlesheet_accounts: &Vec<GoogleSheetAccount>,
) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    // -- Post Sheet Transactions---------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));
    directives
        .extend(post_google_transactions(beancount.clone(), googlesheet_accounts.clone()).await?);

    Ok(directives)
}

async fn post_google_transactions(
    beancount: Beancount,
    googlesheet_accounts: Vec<GoogleSheetAccount>,
) -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();
    let income_accounts = match beancount.user_settings.income.clone() {
        Some(accounts) => accounts,
        None => return Ok(vec![]),
    };

    let asset_accounts = match beancount.user_settings.assets.clone() {
        Some(accounts) => accounts,
        None => return Ok(vec![]),
    };

    for account in googlesheet_accounts {
        let sheet = GoogleSheet::new(account.clone()).await?;

        if let Some(transactions) = sheet.transactions().await {
            for tx in transactions {
                // NOTE: This is a hack to ignore pot transfers and assumes
                // that pot transfers are included separately in `main.beancount`.
                if tx.payment_type == "Pot transfer" {
                    continue;
                }

                let from_posting =
                    match prepare_from_posting(&asset_accounts, &income_accounts, &account, tx) {
                        Ok(posting) => posting,
                        Err(e) => {
                            eprintln!(
                                "Error preparing from posting for account {}: {:?}",
                                account.sheet_name, e
                            );
                            continue;
                        }
                    };

                let to_posting =
                    match prepare_to_posting(&asset_accounts, &income_accounts, &account, tx) {
                        Ok(posting) => posting,
                        Err(e) => {
                            eprintln!(
                                "Error preparing to posting for account {}: {:?}",
                                account.sheet_name, e
                            );
                            continue;
                        }
                    };

                let postings = Postings {
                    from: from_posting,
                    to: to_posting,
                };

                let transaction = prepare_transaction(&postings, tx);

                directives.push(Directive::Transaction(Box::new(transaction)));
            }
        }
    }

    Ok(directives)
}
