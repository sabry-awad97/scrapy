use std::{
    collections::HashSet,
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{
    sync::{mpsc, Barrier},
    time::sleep,
};

use crate::Spider;

use self::{processor::Processor, scraper::Scraper};

pub use crawler_builder::CrawlerBuilder;

mod crawler_builder;
mod processor;
mod scraper;
mod url_processor;

pub struct Crawler {
    active_spiders: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
    crawling_concurrency: usize,
    crawling_queue_capacity: usize,
    delay: Duration,
    processing_concurrency: usize,
    processing_queue_capacity: usize,
}

impl Crawler {
    pub(crate) fn new(
        delay: Duration,
        crawling_concurrency: usize,
        processing_concurrency: usize,
        crawling_queue_capacity: usize,
        processing_queue_capacity: usize,
    ) -> Self {
        let active_spiders = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(3));

        Self {
            active_spiders,
            barrier,
            crawling_concurrency,
            crawling_queue_capacity,
            delay,
            processing_concurrency,
            processing_queue_capacity,
        }
    }

    pub async fn crawl<T, E, S>(&self, spider: S)
    where
        T: Send + 'static,
        E: Display + Send + 'static,
        S: Spider<Item = T, Error = E> + 'static,
    {
        let spider_arc = Arc::new(spider);

        let mut visited_urls = HashSet::<String>::new();

        let (urls_to_visit_tx, urls_to_visit_rx) =
            mpsc::channel::<String>(self.crawling_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(self.processing_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(self.crawling_queue_capacity);

        for url in spider_arc.start_urls() {
            visited_urls.insert(url.clone());
            let _ = urls_to_visit_tx.send(url).await;
        }

        let processor = Processor::new(self.processing_concurrency, self.barrier.clone());
        processor.process_items(spider_arc.clone(), items_rx);

        let scraper = Scraper::new(
            self.active_spiders.clone(),
            self.barrier.clone(),
            self.crawling_concurrency,
            self.delay,
            spider_arc.clone(),
        );

        scraper.scrape_urls(urls_to_visit_rx, new_urls_tx.clone(), items_tx);

        loop {
            if let Ok((visited_url, new_urls)) = new_urls_rx.try_recv() {
                visited_urls.insert(visited_url);

                for url in new_urls {
                    if !visited_urls.contains(&url) {
                        visited_urls.insert(url.clone());
                        log::debug!("queueing: {}", url);
                        let _ = urls_to_visit_tx.send(url).await;
                    }
                }
            }

            if new_urls_tx.capacity() == self.crawling_queue_capacity
                && urls_to_visit_tx.capacity() == self.crawling_queue_capacity
                && self.active_spiders.load(Ordering::SeqCst) == 0
            {
                break;
            }

            sleep(Duration::from_millis(5)).await;
        }

        drop(urls_to_visit_tx);

        self.barrier.wait().await;
    }
}
