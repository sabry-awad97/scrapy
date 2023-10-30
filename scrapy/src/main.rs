use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use log::LevelFilter;
use reqwest::Client;
use scraper::{Html, Selector};
use scrapy::{Crawler, FromHTML, Spider};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Validation Error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone)]
pub struct QuotesItem {
    pub text: Option<String>,
    pub author: Option<String>,
}

impl QuotesItem {
    fn validate(&self) -> Result<(), AppError> {
        if self.text.is_none() {
            return Err(AppError::Validation("Missing text".to_string()));
        }

        if self.author.is_none() {
            return Err(AppError::Validation("Missing author".to_string()));
        }

        Ok(())
    }
}

impl FromHTML for QuotesItem {
    type Error = AppError;
    type Output = Vec<Self>;

    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let document = Html::parse_document(html);
        let quote_selector = Selector::parse(".quote").unwrap();
        let author_selector = Selector::parse("small.author").unwrap();
        let text_selector = Selector::parse("span.text").unwrap();

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

    async fn process(&self, item: Self::Item) -> Result<(), AppError> {
        println!("processing: {:#?}", item);
        item.validate()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_logging();
    let crawler = Crawler::new();

    let spider = Arc::new(QuotesSpider::new());
    crawler.crawl(spider).await;

    Ok(())
}

fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}
