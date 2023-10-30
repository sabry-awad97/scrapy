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

mod processor;
mod scraper;

pub struct Crawler {
    active_spiders: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
    crawling_queue_capacity: usize,
    processing_queue_capacity: usize,
    processor: Processor,
    scraper: Scraper,
}

impl Crawler {
    pub fn new(
        delay: Duration,
        crawling_concurrency: usize,
        processing_concurrency: usize,
    ) -> Self {
        let active_spiders = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(3));
        let crawling_queue_capacity = crawling_concurrency * 400;
        let processing_queue_capacity = processing_concurrency * 10;
        let processor = Processor::new(processing_concurrency, barrier.clone());
        let scraper = Scraper::new(
            active_spiders.clone(),
            barrier.clone(),
            crawling_concurrency,
            delay,
        );

        Self {
            active_spiders,
            barrier,
            crawling_queue_capacity,
            processing_queue_capacity,
            processor,
            scraper,
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

        self.processor.process_items(spider_arc.clone(), items_rx);

        self.scraper.scrape_urls(
            spider_arc.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
        );

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
