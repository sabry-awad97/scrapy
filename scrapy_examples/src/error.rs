#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Spider is not valid: {0}")]
    InvalidSpider(String),
}
