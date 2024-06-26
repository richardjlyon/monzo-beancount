//! Initialise the app folder system and configuration file.
use std::{fs::File, io::Write, path::PathBuf};

use colored::*;
use dialoguer::{Confirm, Input};
use dirs;

use crate::{
    beancount::{Beancount, FilePaths},
    error::AppError as Error,
};

pub async fn init() -> Result<(), Error> {
    println!("Initializing...");

    if !can_initialise() {
        return Err(Error::ApplicationError(
            "Application already initialized".to_string(),
        ));
    }

    let app_folder = get_app_folder_from_user()?;

    let file_paths = match Beancount::initialise_filesystem(app_folder.clone()) {
        Ok(paths) => paths,
        Err(_) => {
            return Err(Error::ApplicationError(
                "Failed to create the necessary folders".to_string(),
            ))
        }
    };

    create_user_config_file(&file_paths)?;
    create_user_beanfiles(&file_paths)?;
    let _ = Beancount::create_app_settings(app_folder)?;

    println!("Initialization complete");

    Ok(())
}

fn can_initialise() -> bool {
    if Beancount::get_app_settings().is_ok() {
        println!("Application already initialized");
        let confirmation = Confirm::new()
            .with_prompt("Do you want to reinitialise?")
            .interact()
            .unwrap();
        if !confirmation {
            return false;
        }
    };

    true
}

fn get_app_folder_from_user() -> Result<PathBuf, Error> {
    let home_dir = dirs::document_dir().unwrap();
    let default_installation_dir = home_dir.join("beancount").to_string_lossy().to_string();
    let root_folder: String = Input::new()
        .with_prompt(&"Folder location?".green().to_string())
        .with_initial_text(default_installation_dir)
        .interact_text()
        .map_err(|e| Error::ApplicationError(e.to_string()))?;

    Ok(PathBuf::from(root_folder))
}

fn create_user_config_file(file_paths: &FilePaths) -> Result<(), Error> {
    let yaml_content = yaml_content(file_paths.root_dir.clone());
    let config_file = file_paths.root_dir.join("beancount.yaml");
    let mut file = File::create(config_file)?;
    file.write_all(yaml_content.as_bytes())?;

    Ok(())
}

fn create_user_beanfiles(file_paths: &FilePaths) -> Result<(), Error> {
    // create a sample include file
    let pension_directives = pension_directives();
    let pension_beanfile = file_paths.include_dir.join("pension-pensionbee.beancount");
    let mut file = File::create(pension_beanfile)?;
    file.write_all(pension_directives.as_bytes())?;

    // create the main file
    let mainfile_directives = mainfile_directives();
    let mainfile = file_paths.root_dir.join("main.beancount");
    let mut file = File::create(mainfile)?;
    file.write_all(mainfile_directives.as_bytes())?;

    Ok(())
}

fn yaml_content(root_path: PathBuf) -> String {
    format!(
        r#"root_dir: {}
start_date: 2024-01-01

assets:

  - account_type: "Assets"
    country: "GBP"
    institution: "Pension"
    account: "BeeHive"

    "#,
        root_path.to_string_lossy()
    )
}

fn pension_directives() -> String {
    r#"2024-01-01 * "Opening Balance"
    Assets:GBP:Pension:BeeHive                    1000.00 GBP
    Equity:OpeningBalances                       -1000.00 GBP

2024-02-01 * "Interest"
  Assets:GBP:Pension:BeeHive                       200.00 GBP
  Income:GBP:Pension:BeeHive                      -200.00 GBP"#
        .to_string()
}

fn mainfile_directives() -> String {
    r#"option "title" "Monzo Accounts"
option "operating_currency" "GBP"
include "include/pension-pensionbee.beancount"

* Asset Accounts

2024-01-01 open Assets:GBP:Pension:BeeHive                         GBP

"#
    .to_string()
}
