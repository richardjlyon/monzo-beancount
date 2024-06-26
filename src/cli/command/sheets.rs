//! Lists available sheets.

use crate::beancount::google::config::load_sheets;
use crate::beancount::google::GoogleSheet;
use crate::error::AppError as Error;

pub async fn sheets() -> Result<(), Error> {
    let sheets = load_sheets()?;
    for sheet in sheets {
        let google_sheet = GoogleSheet::new(sheet).await?;
        println!("{:?}", google_sheet.sheets().await?);
    }

    Ok(())
}
