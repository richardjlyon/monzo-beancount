//! Initialise the app folder system and configuration file.
use std::{env, path::PathBuf};

use colored::Colorize;
use dialoguer::{Confirm, Input};
use dirs;

use crate::{
    beancount::{
        datafile_paths::{DataFilePaths, InitFlag},
        Beancount,
    },
    error::AppError as Error,
};

pub async fn init() -> Result<(), Error> {
    println!("Initializing...");

    // check if the application is already initialized and confirm

    let data_dir = env::var("DATA_DIR")
        .map(PathBuf::from)
        .expect("Failed to get data_dir from .env");

    if Beancount::with_data_dir(data_dir).is_ok() {
        println!("{} Configuration already exists.", "INFO:".green());
        let confirmation = Confirm::new().with_prompt("Continue?").interact().unwrap();

        if !confirmation {
            return Err(Error::ApplicationError(
                "Initialization aborted".to_string(),
            ));
        }
    }

    // try to initialise the data folder

    let data_folder_path = get_data_folder_from_user()?;

    let data_file_paths = match DataFilePaths::with_root(data_folder_path, InitFlag::Initialize) {
        Ok(paths) => paths,
        Err(e) => return Err(e),
    };

    println!("Created paths: {:#?}", data_file_paths);

    // println!("Initialization complete");

    Ok(())
}

fn get_data_folder_from_user() -> Result<PathBuf, Error> {
    let home_dir = dirs::document_dir().unwrap();
    let default_installation_dir = home_dir.join("beancount").to_string_lossy().to_string();
    let root_folder: String = Input::new()
        .with_prompt(&"Folder location?".green().to_string())
        .with_initial_text(default_installation_dir)
        .interact_text()
        .map_err(|e| Error::ApplicationError(e.to_string()))?;

    Ok(PathBuf::from(root_folder))
}
