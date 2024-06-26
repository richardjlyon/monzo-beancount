//! Initialise the app folder system and configuration file.
use std::{fs::File, io::Write, path::Path};

use chrono::NaiveDate;

use crate::{
    beancount::{
        account::{Account, AccountType},
        BeanSettings, Beancount, FilePaths,
    },
    error::AppError as Error,
};

pub async fn init() -> Result<(), Error> {
    println!("Initializing...");

    let file_paths = create_folders()?;
    create_config_file(file_paths.root_dir)?;
    // create_beanfiles();

    // print_instructions();

    Ok(())
}

fn create_folders() -> Result<FilePaths, Error> {
    let home_dir = dirs::home_dir().unwrap();
    let root_dir = home_dir.join("beancount");
    let file_paths = Beancount::initialise_filesystem(root_dir)?;

    Ok(file_paths)
}

fn create_config_file(root_dir: std::path::PathBuf) -> Result<(), Error> {
    let asset_business = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Business".to_string(),
        sub_account: None,
        transaction_id: None,
    };

    let asset_personal = Account {
        account_type: AccountType::Assets,
        country: "GBP".to_string(),
        institution: "Monzo".to_string(),
        account: "Personal".to_string(),
        sub_account: None,
        transaction_id: None,
    };

    let settings = BeanSettings {
        root_dir,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        assets: Some(vec![asset_business, asset_personal]),
        liabilities: None,
        income: None,
        expenses: None,
    };

    let serialized = serde_yaml::to_string(&settings).unwrap();
    let path = Path::new(&settings.root_dir).join("beancount.yaml");
    let mut file = File::create(path).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();

    Ok(())
}
