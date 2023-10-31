use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use futures::stream::StreamExt;
use tokio::sync::{mpsc, Barrier};

use crate::Spider;

use super::config::ScraperConfig;

struct ScraperContext {
    active_spiders: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
}

pub struct Scraper<T, E> {
    config: ScraperConfig,
    context: ScraperContext,
    spider: Arc<dyn Spider<Item = T, Error = E>>,
}

impl<T, E> Scraper<T, E>
where
    T: Send + 'static,
    E: Display + Send + 'static,
{
    pub fn new(
        active_spiders: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
        config: ScraperConfig,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
    ) -> Self {
        Self {
            config,
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
        let active_spiders = self.context.active_spiders.clone();
        let barrier = self.context.barrier.clone();
        let config = self.config;
        let spider = self.spider.clone();

        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(config.crawling_concurrency(), |queued_url| {
                    let queued_url = queued_url.clone();
                    let active_spiders = active_spiders.clone();
                    let items_tx = items_tx.clone();
                    let new_urls_tx = new_urls_tx.clone();
                    let spider = spider.clone();
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
                        tokio::time::sleep(config.delay()).await;
                        active_spiders.fetch_sub(1, Ordering::SeqCst);
                    }
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
