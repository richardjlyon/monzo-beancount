//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

pub mod account;
pub mod directive;
pub mod generate;
pub mod transaction;

use std::{collections::HashMap, path::PathBuf};

use account::Account;
use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::AppError as Error;

/// A struct representing a Beancount file
pub struct Beancount {
    pub file_paths: FilePaths,
    pub start_date: NaiveDate,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub custom_categories: Option<HashMap<String, String>>,
}

/// A struct representing a Beancount configuration file on disk.
#[derive(Debug, Deserialize)]
struct BeanSettings {
    pub root_dir: PathBuf,
    pub start_date: NaiveDate,
    pub custom_categories: Option<HashMap<String, String>>,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
}

/// A struct representing the paths to the Beancount files.
pub(crate) struct FilePaths {
    pub root_dir: PathBuf,
    pub include_dir: PathBuf,
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

        let settings = match cfg.try_deserialize::<BeanSettings>() {
            Ok(settings) => settings,
            Err(e) => {
                println!("{}", e.to_string());
                return Err(Error::ConfigurationError(e));
            }
        };

        let file_paths = Self::initialise_filesystem(settings.root_dir.clone())?;

        Ok(Beancount {
            file_paths,
            start_date: settings.start_date,
            assets: settings.assets,
            liabilities: settings.liabilities,
            income: settings.income,
            custom_categories: settings.custom_categories,
        })
    }

    pub(crate) fn initialise_filesystem(root_dir: PathBuf) -> Result<FilePaths, Error> {
        // create directories
        const INCLUDE_DIR: &str = "include";

        let directory_names: Vec<&str> = vec![INCLUDE_DIR];

        for folder_name in directory_names {
            let directory_path = root_dir.join(folder_name);
            if !directory_path.exists() {
                std::fs::create_dir_all(root_dir.join(directory_path))?;
            }
        }

        // create main file
        let main_file_path = root_dir.join("main.beancount");
        if !main_file_path.exists() {
            std::fs::File::create(&main_file_path)?;
        }

        let include_file_names: Vec<&str> = vec![
            "savings",
            "essential_fixed",
            "essential_variable",
            "discretionary",
        ];

        for file_name in &include_file_names {
            let file_path = root_dir
                .join(INCLUDE_DIR)
                .join(format!("{}.beancount", file_name));
            if !file_path.exists() {
                std::fs::File::create(file_path)?;
            }
        }

        Ok(FilePaths {
            root_dir: root_dir.clone(),
            include_dir: root_dir.join(INCLUDE_DIR),
        })
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config() {
        let beancount = Beancount::from_config();
        assert!(beancount.is_ok());
    }
}
