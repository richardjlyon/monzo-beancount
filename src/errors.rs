use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // -- File error
    #[error("Failed to open file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to parse file: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed parse sheets: {0}")]
    GoogleError(#[from] google_sheets4::Error),
}
