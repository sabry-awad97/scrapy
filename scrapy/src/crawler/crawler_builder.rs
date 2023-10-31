use std::time::Duration;

use crate::Crawler;

pub struct CrawlerBuilder {
    delay: Duration,
    crawling_concurrency: usize,
    processing_concurrency: usize,
    crawling_queue_capacity: Option<usize>,
    processing_queue_capacity: Option<usize>,
}

impl Default for CrawlerBuilder {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(250),
            crawling_concurrency: 2,
            processing_concurrency: 500,
            crawling_queue_capacity: None,
            processing_queue_capacity: None,
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

    pub fn crawling_queue_capacity<O>(mut self, crawling_queue_capacity: O) -> Self
    where
        O: Into<Option<usize>>,
    {
        self.crawling_queue_capacity = crawling_queue_capacity.into();
        self
    }

    pub fn processing_queue_capacity<O>(mut self, processing_queue_capacity: O) -> Self
    where
        O: Into<Option<usize>>,
    {
        self.processing_queue_capacity = processing_queue_capacity.into();
        self
    }

    pub fn build(self) -> Crawler {
        Crawler::new(
            self.delay,
            self.crawling_concurrency,
            self.processing_concurrency,
            self.crawling_queue_capacity
                .unwrap_or(self.crawling_concurrency * 400),
            self.processing_queue_capacity
                .unwrap_or(self.processing_concurrency * 10),
        )
    }
}
