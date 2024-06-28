//! Handles deserialising user settings from the data directory

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::AppError as Error;

use super::{account::Account, google::GoogleSheetAccount};

/// A struct representing a user settings file on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub start_date: NaiveDate,
    pub googlesheet_accounts: Option<Vec<GoogleSheetAccount>>,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub expenses: Option<Vec<Account>>,
}

impl UserSettings {
    /// Constructs a new instance of `UserSettings` from a configuration file.
    pub fn from_config(config_file_path: PathBuf) -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                &config_file_path.to_string_lossy(),
                config::FileFormat::Yaml,
            ))
            .build()?;

        let user_settings = match cfg.try_deserialize::<UserSettings>() {
            Ok(settings) => settings,
            Err(e) => {
                return Err(Error::ConfigurationError(e));
            }
        };

        Ok(user_settings)
    }
}
