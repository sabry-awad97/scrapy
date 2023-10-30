use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use scrapy::FromHTML;
use scrapy::Spider;

use crate::error::AppError;

use super::item::QuotesItem;

pub struct QuotesSpider {
    http_client: Client,
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

    async fn scrape(&self, url: &str) -> Result<(Vec<Self::Item>, Vec<String>), AppError> {
        log::info!("visiting: {}", url);
        let http_res = self.http_client.get(url).send().await?.text().await?;
        let next_pages_link = vec![];
        Ok((Self::Item::from_html(&http_res)?, next_pages_link))
    }

    async fn process(&self, item: Self::Item) -> Result<(), AppError> {
        println!("processing: {:#?}", item);
        item.validate()?;
        Ok(())
    }
}

impl QuotesSpider {
    pub fn new() -> Self {
        let http_timeout = Duration::from_secs(6);
        let http_client = Client::builder()
            .timeout(http_timeout)
            .build()
            .expect("spiders/quotes: Building HTTP client");

        Self { http_client }
    }
}
