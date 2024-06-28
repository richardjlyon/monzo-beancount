//! Manages the generation of Beancount files and related file operations.
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in a Monzo Google sheet.

pub mod account;
pub mod datafile_paths;
pub mod directive;
pub mod generate;
pub mod google;
pub mod transaction;
pub mod user_settings;

use std::path::PathBuf;

use datafile_paths::{DataFilePaths, InitFlag};
use user_settings::UserSettings;

use crate::error::AppError as Error;

/// A struct representing a Beancount file
#[derive(Debug, Clone)]
pub struct Beancount {
    pub data_file_paths: DataFilePaths,
    pub user_settings: UserSettings,
}

/// Constructors
impl Beancount {
    pub fn with_data_dir(data_dir: PathBuf) -> Result<Self, Error> {
        let data_file_paths = DataFilePaths::with_root(data_dir, InitFlag::DoNotInitialize)?;
        let user_settings = UserSettings::from_config(data_file_paths.config_file.clone())?;

        Ok(Self {
            data_file_paths,
            user_settings,
        })
    }
}

/// Associated functions
// impl Beancount {
//     pub fn has_user_settings() -> bool {
//         UserSettings::has_user_settings()
//     }
// }

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn should_return_new_instance() {
        // Determine which .env file to load
        let env_file = env::var("ENV_FILE").unwrap_or_else(|_| ".env.dev".to_string());
        let path = PathBuf::from(env_file);

        // Load environment variables from the specified file
        dotenvy::from_filename(path).expect("Failed to load .env file");

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .expect("Failed to get data_dir from .env");
        let beancount = Beancount::with_data_dir(data_dir);

        assert!(beancount.is_ok());
    }

    #[test]
    fn should_return_configuration_error() {
        let data_dir = PathBuf::from("/tmp");
        let beancount = Beancount::with_data_dir(data_dir);
        match beancount {
            Err(Error::ConfigurationError(_)) => assert!(true),
            _ => panic!("Expected ConfigError, got {:?}", beancount),
        }
    }
}
