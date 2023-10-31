use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::StreamExt;
use tokio::sync::mpsc;

use super::scraper::SpiderScraper;

pub struct UrlProcessor {
    active_spiders: Arc<AtomicUsize>,
    crawling_concurrency: usize,
    delay: Duration,
}

impl UrlProcessor {
    pub fn new(
        active_spiders: Arc<AtomicUsize>,
        crawling_concurrency: usize,
        delay: Duration,
    ) -> Self {
        Self {
            active_spiders,
            crawling_concurrency,
            delay,
        }
    }

    pub async fn process_urls<T, E>(
        &self,
        urls_to_visit: mpsc::Receiver<String>,
        spider_scraper: SpiderScraper<T, E>,
    ) where
        T: Send + 'static,
        E: Display + Send + 'static,
    {
        tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
            .for_each_concurrent(self.crawling_concurrency, |queued_url| {
                let queued_url = queued_url.clone();
                let active_spiders = self.active_spiders.clone();
                let items_tx = spider_scraper.items_tx.clone();
                let new_urls_tx = spider_scraper.new_urls_tx.clone();
                let spider = spider_scraper.spider.clone();
                async move {
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
                    tokio::time::sleep(self.delay).await;
                    active_spiders.fetch_sub(1, Ordering::SeqCst);
                }
            })
            .await;

        drop(spider_scraper.items_tx);
    }
}
