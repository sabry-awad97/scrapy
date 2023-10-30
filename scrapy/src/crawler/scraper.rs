use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::StreamExt;
use tokio::sync::{mpsc, Barrier};

use crate::Spider;

pub struct Scraper {
    active_spiders: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
    crawling_concurrency: usize,
    delay: Duration,
}

impl Scraper {
    pub fn new(
        active_spiders: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
        crawling_concurrency: usize,
        delay: Duration,
    ) -> Self {
        Self {
            active_spiders,
            barrier,
            crawling_concurrency,
            delay,
        }
    }

    pub fn scrape_urls<T, E>(
        &self,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
        urls_to_visit: mpsc::Receiver<String>,
        new_urls_tx: mpsc::Sender<(String, Vec<String>)>,
        items_tx: mpsc::Sender<T>,
    ) where
        T: Send + 'static,
        E: Display + Send + 'static,
    {
        let active_spiders = self.active_spiders.clone();
        let barrier = self.barrier.clone();
        let crawling_concurrency = self.crawling_concurrency;
        let delay = self.delay;

        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(crawling_concurrency, |queued_url| {
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
                        tokio::time::sleep(delay).await;
                        active_spiders.fetch_sub(1, Ordering::SeqCst);
                    }
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
