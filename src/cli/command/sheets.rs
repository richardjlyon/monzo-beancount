//! Lists available sheets.

use crate::beancount::google::GoogleSheet;
use crate::beancount::Beancount;
use crate::error::AppError as Error;

pub async fn sheets(beancount: &Beancount) -> Result<(), Error> {
    match &beancount.user_settings.googlesheet_accounts {
        Some(accounts) => {
            for account in accounts {
                let google_sheet = GoogleSheet::new(account.clone()).await?;
                println!("{:?}", google_sheet.sheets().await?);
            }
        }
        None => {}
    }

    Ok(())
}
