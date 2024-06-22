//! Functionality for accessing a Google Sheet
//!

pub mod config;
pub mod sheets;
pub mod transactions;

use std::{fs::File, io::BufReader};

use config::GoogleAccount;
use google_sheets4::{
    hyper::{self, client::HttpConnector},
    hyper_rustls, oauth2, Sheets,
};
use hyper_rustls::HttpsConnector;
use transactions::Transaction;

use crate::error::AppError;

/// A struct for accessing a Google Sheet.
pub struct GoogleSheet {
    pub hub: Sheets<HttpsConnector<HttpConnector>>,
    pub account: GoogleAccount,
    pub transactions: Option<Vec<Transaction>>,
}

impl GoogleSheet {
    /// Create an authenticated GoogleSheet instance.
    pub async fn new(account: GoogleAccount) -> Result<Self, AppError> {
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

        let transactions = GoogleSheet::transactions(&hub, &account).await?;

        Ok(GoogleSheet {
            hub,
            account,
            transactions,
        })
    }
}
