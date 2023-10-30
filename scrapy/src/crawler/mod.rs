use std::{
    collections::HashSet,
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::StreamExt;
use tokio::{
    sync::{mpsc, Barrier},
    time::sleep,
};
use tokio_stream::wrappers::ReceiverStream;

use crate::Spider;

/// A concurrent web crawler capable of processing and scraping content in parallel.
pub struct Crawler {
    /// A `Duration` indicating the delay between crawling requests.
    delay: Duration,

    /// A `usize` indicating the maximum number of concurrent crawling tasks.
    crawling_concurrency: usize,

    /// A `usize` indicating the maximum number of concurrent processing tasks.
    processing_concurrency: usize,

    /// An `Arc<Barrier>` providing a synchronization point for tasks.
    barrier: Arc<Barrier>,

    /// An `Arc<AtomicUsize>` for tracking the number of active spider tasks.
    active_spiders: Arc<AtomicUsize>,
}

impl Crawler {
    /// Constructs a new `Crawler` instance.
    ///
    /// # Arguments
    ///
    /// * `delay` - The duration between consecutive crawling requests.
    /// * `crawling_concurrency` - The maximum number of concurrent crawling tasks.
    /// * `processing_concurrency` - The maximum number of concurrent processing tasks.
    ///
    /// # Returns
    ///
    /// A new `Crawler` instance with the specified configurations.
    pub fn new(
        delay: Duration,
        crawling_concurrency: usize,
        processing_concurrency: usize,
    ) -> Self {
        let barrier = Arc::new(Barrier::new(3));
        let active_spiders = Arc::new(AtomicUsize::new(0));
        Self {
            delay,
            crawling_concurrency,
            processing_concurrency,
            barrier,
            active_spiders,
        }
    }

    /// Initiates the crawling process using the provided `Spider`.
    ///
    /// # Arguments
    ///
    /// * `spider` - The `Spider` implementation responsible for crawling, processing, and scraping.
    ///
    /// # Remarks
    ///
    /// This method launches concurrent tasks for crawling, processing, and scraping operations.
    ///
    pub async fn crawl<T, E, S>(&self, spider: S)
    where
        T: Send + 'static,
        E: Display + Send + 'static,
        S: Spider<Item = T, Error = E> + 'static,
    {
        let spider_arc = Arc::new(spider);
        let mut visited_urls = HashSet::<String>::new();
        let crawling_queue_capacity = self.crawling_concurrency * 400;
        let processing_queue_capacity = self.processing_concurrency * 10;

        let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel::<String>(crawling_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(processing_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(crawling_queue_capacity);

        for url in spider_arc.start_urls() {
            visited_urls.insert(url.clone());
            let _ = urls_to_visit_tx.send(url).await;
        }

        self.launch_processors(spider_arc.clone(), items_rx);

        self.launch_scrapers(
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

            if new_urls_tx.capacity() == crawling_queue_capacity
                && urls_to_visit_tx.capacity() == crawling_queue_capacity
                && self.active_spiders.load(Ordering::SeqCst) == 0
            {
                break;
            }

            sleep(Duration::from_millis(5)).await;
        }

        drop(urls_to_visit_tx);

        self.barrier.wait().await;
    }

    /// Launches processors to handle incoming items concurrently.
    ///
    /// # Arguments
    ///
    /// * `spider` - The `Spider` implementation responsible for processing items.
    /// * `items` - The receiver channel for incoming items.
    ///
    /// # Remarks
    ///
    /// This method creates a set of asynchronous tasks to process items concurrently.
    ///
    fn launch_processors<T, E>(
        &self,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
        items: mpsc::Receiver<T>,
    ) where
        T: Send + 'static,
        E: Send + 'static,
    {
        let concurrency = self.processing_concurrency;
        let barrier = self.barrier.clone();
        tokio::spawn(async move {
            ReceiverStream::new(items)
                .for_each_concurrent(concurrency, |item| async {
                    let _ = spider.process(item).await;
                })
                .await;

            barrier.wait().await;
        });
    }

    /// Launches scrapers to retrieve content from URLs concurrently.
    ///
    /// # Arguments
    ///
    /// * `spider` - The `Spider` implementation responsible for scraping URLs.
    /// * `urls_to_visit` - The receiver channel for URLs to be visited.
    /// * `new_urls_tx` - The sender channel for new URLs and corresponding scraped content.
    /// * `items_tx` - The sender channel for processed items.
    ///
    /// # Remarks
    ///
    /// This method creates a set of asynchronous tasks to scrape URLs concurrently.
    ///
    fn launch_scrapers<T, E>(
        &self,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
        urls_to_visit: mpsc::Receiver<String>,
        new_urls_tx: mpsc::Sender<(String, Vec<String>)>,
        items_tx: mpsc::Sender<T>,
    ) where
        T: Send + 'static,
        E: Display + Send + 'static,
    {
        let concurrency = self.crawling_concurrency;
        let barrier = self.barrier.clone();
        let delay = self.delay;
        let active_spiders = self.active_spiders.clone();

        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(concurrency, |queued_url| {
                    let queued_url = queued_url.clone();
                    async {
                        active_spiders.fetch_add(1, Ordering::SeqCst);
                        let mut urls = Vec::new();
                        let res = spider.scrape(&queued_url.clone()).await.map_err(|err| {
                            log::error!("{}", err);
                            err
                        });

                        if let Ok((items, new_urls)) = res {
                            for item in items {
                                let _ = items_tx.send(item).await;
                            }
                            urls = new_urls;
                        }

                        let _ = new_urls_tx.send((queued_url, urls)).await;
                        sleep(delay).await;
                        active_spiders.fetch_sub(1, Ordering::SeqCst);
                    }
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
