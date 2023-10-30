use log::LevelFilter;
use reqwest::Client;
use scraper::error::SelectorErrorKind;
use scraper::{Html, Selector};
#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Selector Error: {0}")]
    Selector(#[from] SelectorErrorKind<'static>),

    #[error("Validation Error: {0}")]
    Validation(String),
}

#[derive(Debug)]
pub struct QuotesItem {
    pub text: Option<String>,
    pub author: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_logging();

    let url = "https://quotes.toscrape.com/page/1/";
    let http_client = Client::builder().build().expect("Building HTTP client");

    log::info!("visiting: {}", url);
    let html = http_client.get(url).send().await?.text().await?;

    let document = Html::parse_document(&html);
    let quote_selector = Selector::parse(".quote")?;
    let author_selector = Selector::parse("small.author")?;
    let text_selector = Selector::parse("span.text")?;

    let mut quotes = Vec::new();

    for quote in document.select(&quote_selector) {
        let text: Option<_> = quote
            .select(&text_selector)
            .next()
            .and_then(|e| e.inner_html().parse().ok());

        let author: Option<_> = quote
            .select(&author_selector)
            .next()
            .and_then(|e| e.inner_html().parse().ok());

        quotes.push(QuotesItem { text, author })
    }

    println!("{quotes:#?}");

    for quote in &quotes {
        validate_quote(quote)?;
    }

    Ok(())
}

fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

fn validate_quote(quote: &QuotesItem) -> Result<(), AppError> {
    if quote.text.is_none() {
        return Err(AppError::Validation("Missing text".to_string()));
    }

    if quote.author.is_none() {
        return Err(AppError::Validation("Missing author".to_string()));
    }

    Ok(())
}
