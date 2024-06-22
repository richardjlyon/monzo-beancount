//! Update transactions command

use crate::error::AppError as Error;
use crate::google::{load_ids, GoogleSheet};

pub async fn update() -> Result<(), Error> {
    let bean = Beancount::from_config()?;
    let google = GoogleSheet::new().await?;
    let sheet = load_ids()?;

    let transactions = google.transactions(&sheet.personal).await?.unwrap();

    // -- Initialise the file system -----------------------------------------------------

    match bean.initialise_filesystem()? {
        Some(message) => println!("{}", message),
        None => {}
    };

    }

    Ok(())
}
