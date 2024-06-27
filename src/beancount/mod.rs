//! Manages the generation of Beancount files and related file operations.
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in a Monzo Google sheet.

pub mod account;
pub mod directive;
pub mod generate;
pub mod google;
pub mod transaction;

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use account::Account;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::AppError as Error;

/// A struct representing a Beancount file
#[derive(Debug)]
pub struct Beancount {
    pub user_settings: UserSettings,
    pub file_paths: FilePaths,
}

/// A struct representing a Beancount configuration file on disk
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings {
    pub start_date: NaiveDate,
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

/// A struct respresenting Beancount configuration settins
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationSettings {
    /// The directory of the Beancount folder
    root_dir: PathBuf,
}

/// Constructors
impl Beancount {
    /// Create a new Beancount instance from a user configuration file
    ///
    /// # Errors
    /// Will return an error if the configuration file cannot be read
    pub fn from_user_config() -> Result<Self, Error> {
        let application_settings = match Beancount::get_app_settings() {
            Ok(app_settings) => app_settings,
            Err(e) => return Err(Error::ApplicationError(e.to_string())),
        };

        let home_folder = application_settings.root_dir.clone();
        let file_paths = Self::initialise_filesystem(application_settings.root_dir.clone())?;
        let user_settings = get_user_settings(home_folder)?;

        Ok(Beancount {
            file_paths,
            user_settings,
        })
    }

    // Get application_settings from the Application Support directory.
    pub fn get_app_settings() -> Result<ApplicationSettings, Error> {
        let settings_filepath = construct_app_settings_filepath()?;

        if !settings_filepath.exists() {
            return Err(Error::ApplicationError(
                "Setting folder doesn't exist. Run command `init`".to_string(),
            ));
        };

        let cfg = config::Config::builder()
            .add_source(config::File::new(
                &settings_filepath.to_string_lossy(),
                config::FileFormat::Yaml,
            ))
            .build()?;

        let app_settings = match cfg.try_deserialize::<ApplicationSettings>() {
            Ok(settings) => settings,
            Err(e) => {
                println!("{}", e);
                return Err(Error::ConfigurationError(e));
            }
        };

        Ok(app_settings)
    }

    // Create the application settings file in the Application Support directory.
    pub fn create_app_settings(root_dir: PathBuf) -> Result<ApplicationSettings, Error> {
        let settings_filepath = construct_app_settings_filepath()?;
        let settings = ApplicationSettings { root_dir };
        let serialized = serde_yaml::to_string(&settings)?;
        let mut file = File::create(settings_filepath.clone())?;
        file.write_all(serialized.as_bytes())?;

        Ok(settings)
    }

    // Initialise the user filesystem
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

        // save then root folder to the config file

        Ok(file_paths)
    }
}

// create the filepath to the settings file in the Application Support directory
fn construct_app_settings_filepath() -> Result<PathBuf, Error> {
    let application_support_dir = dirs::config_dir().ok_or_else(|| {
        Error::ApplicationError("Couldn't open system Application Support directory".to_string())
    })?;
    let application_dir = application_support_dir.join("Beancount");
    fs::create_dir_all(application_dir.clone())?;

    let settings_file = application_dir.join("settings.yaml");

    Ok(settings_file)
}

// get user settings from the user configuration file
fn get_user_settings(home_folder: PathBuf) -> Result<UserSettings, Error> {
    let settings_file = home_folder
        .join("beancount.yaml")
        .to_string_lossy()
        .to_string();

    let cfg = config::Config::builder()
        .add_source(config::File::new(&settings_file, config::FileFormat::Yaml))
        .build()?;

    let user_settings = match cfg.try_deserialize::<UserSettings>() {
        Ok(settings) => settings,
        Err(e) => {
            println!("{}", e);
            return Err(Error::ConfigurationError(e));
        }
    };

    Ok(user_settings)
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_construct_from_config() {
        let beancount = Beancount::from_user_config();
        println!("{:#?}", beancount);
        // assert!(beancount.is_ok());
    }

    #[test]
    fn should_initialise_filesystem() {
        let test_dir = temp_dir::TempDir::with_prefix("monzo-test").unwrap();
        let file_paths = Beancount::initialise_filesystem(test_dir.path().to_path_buf());
        assert!(file_paths.is_ok());
    }
}
