use std::time::Duration;

use crate::Crawler;

pub struct CrawlerBuilder {
    delay: Duration,
    crawling_concurrency: usize,
    processing_concurrency: usize,
}

impl Default for CrawlerBuilder {
    fn default() -> Self {
        Self {
            delay: Duration::from_secs(1),
            crawling_concurrency: 1,
            processing_concurrency: 1,
        }
    }
}

impl CrawlerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn crawling_concurrency(mut self, crawling_concurrency: usize) -> Self {
        self.crawling_concurrency = crawling_concurrency;
        self
    }

    pub fn processing_concurrency(mut self, processing_concurrency: usize) -> Self {
        self.processing_concurrency = processing_concurrency;
        self
    }

    pub fn build(self) -> Crawler {
        Crawler::new(
            self.delay,
            self.crawling_concurrency,
            self.processing_concurrency,
        )
    }
}
