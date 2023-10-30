use std::{collections::HashSet, fmt::Display, sync::Arc};

use tokio::time::Duration;

use crate::Spider;

pub struct Crawler {}

impl Crawler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn crawl<I, E>(&self, spider: Arc<dyn Spider<Item = I, Error = E>>)
    where
        I: Send,
        E: Send + Display,
    {
        let mut visited_urls = HashSet::<String>::new();

        for url in spider.start_urls().iter() {
            visited_urls.insert(url.into());
            if let Ok(items) = spider.scrape(url).await.map_err(|err| {
                log::error!("{}", err);
                err
            }) {
                for item in items {
                    let _ = spider.process(item).await;
                }
            }
        }
    }
}
