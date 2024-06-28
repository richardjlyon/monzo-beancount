//! Handles creation of file paths for the data directory.

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::error::AppError as Error;

/// A struct representing paths to directories and files in the data directory.
#[derive(Debug, Clone)]
pub struct DataFilePaths {
    pub data_dir: PathBuf,
    pub include_dir: PathBuf,
    pub import_dir: PathBuf,
    pub main_file: PathBuf,
    pub config_file: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum InitFlag {
    Initialize,
    DoNotInitialize,
}

impl DataFilePaths {
    /// Constructs a new instance of `FilePaths` with the given path root.
    pub fn with_root(data_dir: PathBuf, init_flag: InitFlag) -> Result<Self, Error> {
        const INCLUDE_DIR: &str = "include";
        const IMPORT_DIR: &str = "import";

        let include_dir = data_dir.join(INCLUDE_DIR);
        let import_dir = data_dir.join(IMPORT_DIR);

        const MAINFILE_NAME: &str = "main.beancount";
        const CONFIG_FILE_NAME: &str = "beancount.yaml";

        let main_file = data_dir.join(MAINFILE_NAME);
        let config_file = data_dir.join(CONFIG_FILE_NAME);

        if let InitFlag::Initialize = init_flag {
            // create directtories
            std::fs::create_dir_all(&include_dir)?;
            std::fs::create_dir_all(&import_dir)?;
            std::fs::write(&main_file, "")?;

            // if config_file doesn't exist, create it
            if !config_file.exists() {
                let mut file = fs::File::create(&config_file)?;
                let config_file_yaml = get_config_file_yaml();
                writeln!(file, "{}", config_file_yaml)?;
            }

            // update .env
            let env_path = get_env_path().unwrap_or_default();
            let mut file = fs::File::create(&env_path)?;
            writeln!(file, "DATA_DIR=\"{}\"", data_dir.to_string_lossy())?;
        }

        Ok(DataFilePaths {
            data_dir,
            include_dir,
            import_dir,
            main_file,
            config_file,
        })
    }
}

fn get_env_path() -> Option<PathBuf> {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let mut dir = current_dir.as_path();

    while dir != Path::new("/") {
        if dir.join(".env").exists() {
            let folder_path = dir.to_path_buf();
            return Some(folder_path.join(".env"));
        }
        dir = dir.parent()?;
    }

    None
}

fn get_config_file_yaml() -> &'static str {
    r#"start_date: "2024-01-01"
googlesheet_accounts:
  - country: "GBP"
    institution: "Monzo"
    name: "personal"
    sheet_name: "Personal Account Transactions"
    sheet_id: "XXX"

  - country: "GBP"
    institution: "Monzo"
    name: "business"
    sheet_name: "Business Account Transactions"
    sheet_id: "XXX"

assets:

"#
}
