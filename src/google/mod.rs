//! Functionality for accessing a Google Sheet
//!

pub mod transactions;

use std::{fs::File, io::BufReader};

use google_sheets4::{
    hyper::{self, client::HttpConnector},
    hyper_rustls, oauth2, Sheets,
};
use hyper_rustls::HttpsConnector;
use serde::Deserialize;

use crate::errors::AppError;

/// A struct for accessing a Google Sheet.
pub struct GoogleSheet {
    pub hub: Sheets<HttpsConnector<HttpConnector>>,
}

impl GoogleSheet {
    /// Create an authenticated GoogleSheet instance.
    pub async fn new() -> Result<Self, AppError> {
        let secret_file = File::open("credentials.json").unwrap();
        let reader = BufReader::new(secret_file);
        let secret: oauth2::ApplicationSecret = serde_json::from_reader(reader).unwrap();

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await?;

        let hub = Sheets::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        Ok(GoogleSheet { hub })
    }

    pub async fn get_sheet_names(&self, id: &str) -> Result<Option<Vec<String>>, AppError> {
        let result = self.hub.spreadsheets().get(id).doit().await?;

        let sheets = match result.1.sheets {
            Some(sheets) => Some(
                sheets
                    .iter()
                    .filter_map(|sheet| sheet.properties.as_ref().and_then(|p| p.title.as_ref()))
                    .map(|title| title.to_string())
                    .collect(),
            ),
            None => None,
        };

        Ok(sheets)
    }
}

/// A struct for the sheet id and name
#[derive(Debug, Deserialize)]
pub struct SheetConfig {
    pub personal: SheetDetails,
    pub business: SheetDetails,
}

#[derive(Debug, Deserialize)]
pub struct SheetDetails {
    pub id: String,
    pub name: String,
}

/// Load sheet id and name from the config file.
pub fn load_ids() -> Result<SheetConfig, AppError> {
    let file = File::open("sheet_ids.yaml")?;
    let reader = BufReader::new(file);
    let config = serde_yaml::from_reader(reader)?;
    Ok(config)
}
