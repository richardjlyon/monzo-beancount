//! Authenticates and accesses a Google Sheet.
//!

pub mod expense_accounts;
pub mod sheets;
pub mod transactions;

use std::{fs::File, io::BufReader};

use google_sheets4::{
    hyper::{self, client::HttpConnector},
    hyper_rustls, oauth2, Sheets,
};
use hyper_rustls::HttpsConnector;
use serde::{Deserialize, Serialize};
use transactions::Transaction;

use crate::error::AppError as Error;

/// A struct for representing a Google Sheet.
pub struct GoogleSheet {
    pub hub: Sheets<HttpsConnector<HttpConnector>>,
    pub account: GoogleSheetAccount,
    pub transactions: Option<Vec<Transaction>>,
}

/// A struct to represent a Google Sheet account
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleSheetAccount {
    pub country: String,
    pub institution: String,
    pub name: String,
    pub sheet_name: String,
    pub sheet_id: String,
}
impl GoogleSheet {
    /// Create an authenticated GoogleSheet instance.
    pub async fn new(account: GoogleSheetAccount) -> Result<Self, Error> {
        let secret_file = File::open("/data/credentials.json").unwrap();
        let reader = BufReader::new(secret_file);
        let secret: oauth2::ApplicationSecret = serde_json::from_reader(reader).unwrap();

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("/data/tokencache.json")
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

        let transactions = GoogleSheet::load_transactions(&hub, &account).await?;

        Ok(GoogleSheet {
            hub,
            account,
            transactions,
        })
    }
}
