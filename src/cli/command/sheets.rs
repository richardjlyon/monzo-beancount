//! Sheets command

use crate::error::AppError as Error;
use crate::google::{load_sheets, GoogleSheet};

pub async fn sheets() -> Result<(), Error> {
    let sheet = load_sheets()?;
    let personal = GoogleSheet::new(sheet.personal).await?;
    let business = GoogleSheet::new(sheet.business).await?;

    println!("{:?}", personal.sheets().await?);
    println!("{:?}", business.sheets().await?);

    Ok(())
}
