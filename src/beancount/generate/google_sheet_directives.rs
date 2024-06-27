//! Process Google Sheet inputs and generate transaction directives.

use crate::beancount::google::config::load_sheets;
use crate::beancount::google::GoogleSheet;
use crate::beancount::{directive::Directive, transaction::Postings};
use crate::error::AppError as Error;

use super::{prepare_from_posting, prepare_to_posting, prepare_transaction};
use futures::future::join_all;
use tokio::task;

pub(crate) async fn google_sheet_directives() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();

    // -- Post Sheet Transactions---------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));
    directives.extend(post_google_transactions().await?);

    Ok(directives)
}

async fn post_google_transactions() -> Result<Vec<Directive>, Error> {
    let mut directives: Vec<Directive> = Vec::new();
    let accounts = load_sheets()?;

    let mut handles = Vec::new();

    for account in accounts {
        let handle = task::spawn(async move {
            let sheet = GoogleSheet::new(account.clone()).await?;
            let mut local_directives = Vec::new();

            if let Some(transactions) = sheet.transactions().await {
                for tx in transactions {
                    // NOTE: This is a hack to ignore pot transfers and assumes
                    // that pot transfers are included separately in `main.beancount`.
                    if tx.payment_type == "Pot transfer" {
                        continue;
                    }

                    let from_posting = match prepare_from_posting(&account, tx) {
                        Ok(posting) => posting,
                        Err(e) => {
                            eprintln!(
                                "Error preparing from posting for account {}: {:?}",
                                account.sheet_name, e
                            );
                            continue;
                        }
                    };

                    let to_posting = match prepare_to_posting(&account, tx) {
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

                    local_directives.push(Directive::Transaction(Box::new(transaction)));
                }
            }

            Ok::<Vec<Directive>, Error>(local_directives)
        });

        handles.push(handle);
    }

    let results = join_all(handles).await;

    for result in results {
        match result {
            Ok(Ok(local_directives)) => {
                directives.extend(local_directives);
            }
            Ok(Err(e)) => {
                eprintln!("Error processing account: {:?}", e);
            }
            Err(e) => {
                eprintln!("Task failed: {:?}", e);
            }
        }
    }

    Ok(directives)
}
