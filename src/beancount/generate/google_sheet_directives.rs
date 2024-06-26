//! Process Google Sheet inputs and generate transaction directives.

use crate::beancount::google::config::load_sheets;
use crate::beancount::google::GoogleSheet;
use crate::beancount::{directive::Directive, transaction::Postings};
use crate::error::AppError as Error;

use super::{prepare_from_posting, prepare_to_posting, prepare_transaction};

pub(crate) async fn google_sheet_directives() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    println!("creating google sheet directives");

    // -- Post Sheet Transactions---------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));
    directives.extend(post_google_transactions().await?);

    Ok(directives)
}

async fn post_google_transactions() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let accounts = load_sheets()?;

    for account in accounts {
        let sheet = GoogleSheet::new(account.clone()).await?;

        if let Some(transactions) = sheet.transactions() {
            for tx in transactions {
                // NOTE: This is a hack to ignore pot transfers and assumes
                // that pot transfers are processed separately.
                if tx.payment_type == "Pot transfer" {
                    continue;
                }

                let from_posting = prepare_from_posting(&account, tx)?;
                let to_posting = prepare_to_posting(&account, tx)?;

                let postings = Postings {
                    from: from_posting,
                    to: to_posting,
                };

                let transaction = prepare_transaction(&postings, tx);

                directives.push(Directive::Transaction(transaction));
            }
        }
    }

    Ok(directives)
}
