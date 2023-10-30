use std::{sync::Arc, fmt::Display};

use futures::StreamExt;
use tokio::sync::{mpsc, Barrier};
use tokio_stream::wrappers::ReceiverStream;

use crate::Spider;

pub struct Processor {
    processing_concurrency: usize,
    barrier: Arc<Barrier>,
}

impl Processor {
    pub fn new(processing_concurrency: usize, barrier: Arc<Barrier>) -> Self {
        Self {
            processing_concurrency,
            barrier,
        }
    }

    pub fn process_items<T, E>(
        &self,
        spider: Arc<dyn Spider<Item = T, Error = E>>,
        items_rx: mpsc::Receiver<T>,
    ) where
        T: Send + 'static,
        E: Display + 'static,
    {
        let processing_concurrency = self.processing_concurrency;
        let barrier = self.barrier.clone();
        tokio::spawn(async move {
            ReceiverStream::new(items_rx)
                .for_each_concurrent(processing_concurrency, |item| async {
                    let _ = spider.process(item).await.map_err(|err| {
                        log::error!("{}", err);
                    });
                })
                .await;

            barrier.wait().await;
        });
    }
}
