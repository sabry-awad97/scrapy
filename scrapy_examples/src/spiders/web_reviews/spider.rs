use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use scrapy::Spider;
use serde_json::json;
use thirtyfour::{ChromeCapabilities, WebDriver};
use tokio::sync::RwLock;

use crate::error::AppError;

#[derive(Clone)]
pub struct WebReviewsSpider {
    driver: Arc<RwLock<WebDriver>>,
    item_index: Arc<AtomicUsize>,
}

#[async_trait]
impl Spider for WebReviewsSpider {
    type Item = HashMap<String, String>;
    type Error = AppError;

    fn name(&self) -> String {
        String::from("web-reviews")
    }

    fn start_urls(&self) -> Vec<String> {
        vec!["https://www.sephora.nz/products/the-ordinary-niacinamide-10-percent-plus-zinc-1-percent/v/30ml".to_string()]
    }

    async fn scrape(&self, url: &str) -> Result<(Vec<Self::Item>, Vec<String>), AppError> {
        log::info!("Visiting: {}", url);

        let webdriver = self.driver.read().await;

        webdriver.goto(&url).await?;
        let html = webdriver.source().await?;

        println!("next_button: {}", html);

        Ok((vec![], vec![]))
    }

    async fn process(&self, item: Self::Item) -> Result<(), AppError> {
        let i = self.item_index.load(Ordering::SeqCst);
        log::info!("Processing: {}. {:?}", i + 1, item);
        self.item_index.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl WebReviewsSpider {
    pub async fn new(headless: bool) -> Result<Self, AppError> {
        let mut caps = ChromeCapabilities::new();

        if headless {
            caps.add_chrome_arg("--headless")?;
        }

        caps.add_chrome_arg("--enable-automation")?;
        caps.add_chrome_arg("--no-sandbox")?;
        caps.add_chrome_arg("--disable-dev-shm-usage")?;
        caps.add_chrome_arg("--disable-gpu")?;

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
            driver: Arc::new(RwLock::new(driver)),
            item_index: AtomicUsize::new(0).into(),
        })
    }

    pub async fn close(&self) -> Result<(), AppError> {
        let driver = self.driver.read().await;
        driver.clone().quit().await?;
        Ok(())
    }
}
