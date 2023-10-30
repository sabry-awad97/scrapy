use std::time::Duration;

use async_trait::async_trait;
use log::LevelFilter;
use reqwest::Client;
use scraper::{error::SelectorErrorKind, Html, Selector};
use scrapy::{FromHTML, Spider};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Selector Error: {0}")]
    Selector(#[from] SelectorErrorKind<'static>),

    #[error("Validation Error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone)]
pub struct QuotesItem {
    pub text: Option<String>,
    pub author: Option<String>,
}

impl FromHTML for QuotesItem {
    type Error = AppError;
    type Output = Vec<Self>;

    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let document = Html::parse_document(html);
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

        Ok(quotes)
    }
}

pub struct QuotesSpider {
    http_client: Client,
}

impl QuotesSpider {
    fn new() -> Self {
        let http_timeout = Duration::from_secs(6);
        let http_client = Client::builder()
            .timeout(http_timeout)
            .build()
            .expect("spiders/quotes: Building HTTP client");

        Self { http_client }
    }
}

#[async_trait]
impl Spider for QuotesSpider {
    type Item = QuotesItem;
    type Error = AppError;

    fn name(&self) -> String {
        String::from("quotes")
    }

    fn start_urls(&self) -> Vec<String> {
        vec![
            "https://quotes.toscrape.com/page/1/".to_string(),
            "https://quotes.toscrape.com/page/2/".to_string(),
        ]
    }

    async fn scrape(&self, url: &str) -> Result<Vec<Self::Item>, Self::Error> {
        log::info!("visiting: {}", url);
        let http_res = self.http_client.get(url).send().await?.text().await?;
        Ok(QuotesItem::from_html(&http_res)?)
    }

    async fn process(&self, _: Self::Item) -> Result<(), AppError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_logging();

    let url = "https://quotes.toscrape.com/page/1/";
    let spider = QuotesSpider::new();
    let quotes = spider.scrape(url).await?;

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
