use std::time::Duration;

#[derive(Clone, Copy)]
pub struct ScraperConfig {
    crawling_concurrency: usize,
    delay: Duration,
}

impl ScraperConfig {
    pub fn new(crawling_concurrency: usize, delay: Duration) -> Self {
        Self {
            crawling_concurrency,
            delay,
        }
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }

    pub fn crawling_concurrency(&self) -> usize {
        self.crawling_concurrency
    }
}
