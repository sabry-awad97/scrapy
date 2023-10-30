use std::{sync::Arc, time::Duration};

use error::AppError;
use log::LevelFilter;
use scrapy::Crawler;
use spiders::QuotesSpider;

mod error;
mod spiders;

fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    setup_logging();
    
    let crawler = Crawler::new(Duration::from_millis(200), 2, 500);

    let spider = Arc::new(QuotesSpider::new());
    crawler.crawl(spider).await;

    Ok(())
}
