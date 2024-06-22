//! Sheets command

use crate::error::AppError as Error;
use crate::google::{load_ids, GoogleSheet};

pub async fn sheets() -> Result<(), Error> {
    let google = GoogleSheet::new().await?;
    let sheet = load_ids()?;

    let sheets = google.get_sheet_names(&sheet.personal.id).await?;
    println!("{:?}", sheets);

    let sheets = google.get_sheet_names(&sheet.business.id).await?;
    println!("{:?}", sheets);

    Ok(())
}
