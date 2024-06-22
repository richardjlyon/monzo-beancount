//! Update transactions command

use crate::error::AppError as Error;
use crate::google::{load_ids, GoogleSheet};

pub async fn update() -> Result<(), Error> {
    let google = GoogleSheet::new().await?;
    let sheet = load_ids()?;

    println!("Update transactions");
    let transactions = google.transactions(&sheet.personal).await?.unwrap();

    for tx in transactions {
        println!("{:#?}", tx);
    }

    Ok(())
}
