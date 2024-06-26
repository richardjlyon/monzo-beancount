//! Manages the generation of Beancount files and related file operations.
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in a Monzo Google sheet.

pub mod account;
pub mod directive;
pub mod generate;
pub mod google;
pub mod transaction;

use std::path::PathBuf;

use account::Account;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::AppError as Error;

/// A struct representing a Beancount file
#[derive(Debug)]
pub struct Beancount {
    pub start_date: NaiveDate,
    pub file_paths: FilePaths,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub expenses: Option<Vec<Account>>,
}

/// A struct representing the paths to the Beancount files
#[derive(Debug)]
pub(crate) struct FilePaths {
    pub main_file: PathBuf,
    pub root_dir: PathBuf,
    pub include_dir: PathBuf,
    pub import_dir: PathBuf,
}

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Serialize, Deserialize)]
pub struct BeanSettings {
    pub root_dir: PathBuf,
    pub start_date: NaiveDate,
    pub assets: Option<Vec<Account>>,
    pub liabilities: Option<Vec<Account>>,
    pub income: Option<Vec<Account>>,
    pub expenses: Option<Vec<Account>>,
}

/// Constructors
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
                println!("{}", e);
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
            expenses: settings.expenses,
        })
    }
}

/// File handling functions
impl Beancount {
    pub fn initialise_filesystem(root_dir: PathBuf) -> Result<FilePaths, Error> {
        // create directories
        const INCLUDE_DIR: &str = "include";
        const IMPORT_DIR: &str = "import";

        let directory_names: Vec<&str> = vec![INCLUDE_DIR, IMPORT_DIR];

        for folder_name in directory_names {
            let directory_path = root_dir.join(folder_name);
            if !directory_path.exists() {
                std::fs::create_dir_all(root_dir.join(directory_path))?;
            }
        }

        // create `main.beancount` if it doesn't exist
        let main_file_path = root_dir.join("main.beancount");
        if !main_file_path.exists() {
            std::fs::File::create(&main_file_path)?;
        }

        let file_paths = FilePaths {
            main_file: main_file_path,
            root_dir: root_dir.clone(),
            include_dir: root_dir.join(INCLUDE_DIR),
            import_dir: root_dir.join(IMPORT_DIR),
        };

        Ok(file_paths)
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_construct_from_config() {
        let beancount = Beancount::from_config();
        assert!(beancount.is_ok());
    }

    #[test]
    fn should_initialise_filesystem() {
        let test_dir = temp_dir::TempDir::with_prefix("monzo-test").unwrap();
        let file_paths = Beancount::initialise_filesystem(test_dir.path().to_path_buf());
        assert!(file_paths.is_ok());
    }
}
