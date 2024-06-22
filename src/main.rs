mod beancount;
mod error;
mod google;

use error::AppError;

use google::{load_ids, GoogleSheet};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let google = GoogleSheet::new().await?;
    let sheet = load_ids()?;

    // Process and print the sheet names
    if let Some(sheets) = google.get_sheet_names(&sheet.personal.id).await? {
        println!("{:?}", sheets);
    };

    let transactions = google.transactions(&sheet.personal).await?.unwrap();

    for tx in transactions {
        println!("{:#?}", tx);
    }

    Ok(())
}
