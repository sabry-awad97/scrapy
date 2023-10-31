use std::{
    fmt::Display,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use tokio::sync::{mpsc, Barrier};

use crate::Spider;

use super::url_processor::UrlProcessor;

struct ScraperContext {
    active_spiders: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
}

pub struct Scraper<T, E> {
    crawling_concurrency: usize,
    delay: Duration,
    context: ScraperContext,
    spider: Arc<dyn Spider<Item = T, Error = E>>,
}

pub struct SpiderScraper<T, E> {
    pub spider: Arc<dyn Spider<Item = T, Error = E>>,
    pub items_tx: mpsc::Sender<T>,
    pub new_urls_tx: mpsc::Sender<(String, Vec<String>)>,
}

impl<T, E> Scraper<T, E>
where
    T: Send + 'static,
    E: Display + Send + 'static,
{
    pub fn new(
        active_spiders: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
        crawling_concurrency: usize,
        delay: Duration,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
    ) -> Self {
        Self {
            crawling_concurrency,
            delay,
            context: ScraperContext {
                active_spiders,
                barrier,
            },
            spider,
        }
    }

    pub fn scrape_urls(
        &self,
        urls_to_visit: mpsc::Receiver<String>,
        new_urls_tx: mpsc::Sender<(String, Vec<String>)>,
        items_tx: mpsc::Sender<T>,
    ) {
        let url_processor = UrlProcessor::new(
            self.context.active_spiders.clone(),
            self.crawling_concurrency,
            self.delay,
        );

        let spider_scraper = SpiderScraper {
            spider: self.spider.clone(),
            items_tx,
            new_urls_tx,
        };

        let barrier = self.context.barrier.clone();

        tokio::spawn(async move {
            url_processor
                .process_urls(urls_to_visit, spider_scraper)
                .await;
            barrier.wait().await;
        });
    }
}
