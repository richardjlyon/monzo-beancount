//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

pub mod account;
pub mod directive;
pub mod expense;
pub mod transaction;

use chrono::NaiveDate;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

// use account::Account;
use crate::error::AppError as Error;
use expense::Expense;

pub use account::{Account, AccountType};
pub use transaction::Transaction;

/// A struct representing a Beancount file
pub struct Beancount {
    pub settings: BeanSettings,
}

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Deserialize)]
pub struct BeanSettings {
    pub root_dir: PathBuf,
    pub csv_dir: PathBuf,
    pub start_date: NaiveDate,
    pub custom_categories: Option<HashMap<String, String>>,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub expenses: Option<Vec<Expense>>,
}

impl Beancount {
    /// Create a new Beancount instance
    ///
    /// # Errors
    /// Will return an error if the configuration file cannot be read
    pub fn from_config() -> Result<Self, Error> {
        let cfg = config::Config::builder()
            .add_source(config::File::new(
                "beancount.yaml",
                config::FileFormat::Yaml,
            ))
            .build()?;

        match cfg.try_deserialize::<BeanSettings>() {
            Ok(settings) => Ok(Beancount { settings }),
            Err(e) => {
                println!("{}", e.to_string());
                Err(Error::ConfigurationError(e))
            }
        }
    }

    // Iniitialise the file system
    pub fn initialise_filesystem(&self) -> Result<Option<String>, Error> {
        let path = self.settings.root_dir.clone();
        if !path.exists() {
            std::fs::create_dir_all(path.clone())?;
            return Ok(Some(format!(
                "Created beanfile directory at: {}",
                path.to_string_lossy()
            )));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config() {
        let beancount = Beancount::from_config();
        assert!(beancount.is_ok());
    }
}
