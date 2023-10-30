use thirtyfour::prelude::WebDriverError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("WebDriver Error: {0}")]
    WebDriver(#[from] WebDriverError),

    #[error("Validation Error: {0}")]
    Validation(String),

    #[error("Spider is not valid: {0}")]
    InvalidSpider(String),
}
