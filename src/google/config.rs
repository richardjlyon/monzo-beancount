//! Handles the configuration of the application.

use std::{fs::File, io::BufReader};

use serde::Deserialize;

use crate::error::AppError as Error;

/// A struct to represent a Google Sheet account
#[derive(Debug, Deserialize, Clone)]
pub struct GoogleAccount {
    pub country: String,
    pub institution: String,
    pub name: String,
    pub sheet_name: String,
    pub sheet_id: String,
}

/// Load sheet id and name from the config file.
pub fn load_sheets() -> Result<Vec<GoogleAccount>, Error> {
    let file = File::open("google_accounts.yaml")?;
    let reader = BufReader::new(file);
    let config = serde_yaml::from_reader(reader)?;

    Ok(config)
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use crate::google::GoogleSheet;

    use super::*;

    #[tokio::test]
    async fn new() {
        let sheets = load_sheets().unwrap();
        let sheet = sheets[0].clone();

        let personal = GoogleSheet::new(sheet).await.unwrap();

        assert!(personal.transactions.unwrap().len() > 0);
    }
}
