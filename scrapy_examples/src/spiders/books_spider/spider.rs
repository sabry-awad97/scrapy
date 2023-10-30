use std::sync::Arc;

use async_trait::async_trait;
use scrapy::{FromHTML, Spider};
use serde_json::json;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio::sync::Mutex;

use crate::error::AppError;

use super::item::BookItem;

#[derive(Clone)]
pub struct BooksSpider {
    driver: Arc<Mutex<WebDriver>>,
    base_url: String,
}

#[async_trait]
impl Spider for BooksSpider {
    type Item = BookItem;
    type Error = AppError;

    fn name(&self) -> String {
        String::from("books")
    }

    fn start_urls(&self) -> Vec<String> {
        vec![self.base_url.to_string()]
    }

    async fn scrape(&self, url: &str) -> Result<(Vec<Self::Item>, Vec<String>), AppError> {
        log::info!("visiting: {}", url);

        let html = {
            let webdriver = self.driver.lock().await;
            webdriver.goto(&url).await?;
            webdriver.source().await?
        };

        let next_pages_link = vec![];

        Ok((Self::Item::from_html(&html)?, next_pages_link))
    }

    async fn process(&self, item: Self::Item) -> Result<(), AppError> {
        if let Some(title) = item.title {
            println!("Book Title: {:?}", title);
        }

        Ok(())
    }
}

impl BooksSpider {
    pub async fn new(headless: bool) -> Result<Self, AppError> {
        let mut caps = DesiredCapabilities::chrome();

        if headless {
            caps.add_chrome_option("args", ["--headless", "--disable-gpu"])?;
        }

        caps.add_chrome_option(
            "prefs",
            json!({
                "profile.default_content_settings": {
                    "images": 2 // Do not load images.
                },
                "profile.managed_default_content_settings": {
                    "images": 2 // Do not load images.
                }
            }),
        )?;
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        Ok(Self {
            driver: Arc::new(Mutex::new(driver)),
            base_url: "http://books.toscrape.com".to_string(),
        })
    }

    pub async fn close(&self) -> Result<(), AppError> {
        let driver = self.driver.lock().await;
        driver.clone().quit().await?;
        Ok(())
    }

    #[allow(unused)]
    fn normalize_url(&self, url: &str) -> String {
        let url = url.trim();

        if url.starts_with('/') {
            return format!("{}{}", self.base_url, url);
        }

        url.to_string()
    }
}
