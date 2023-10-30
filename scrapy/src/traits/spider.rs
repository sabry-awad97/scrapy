use async_trait::async_trait;

#[async_trait]
pub trait Spider: Send + Sync {
    type Item;
    type Error;

    fn name(&self) -> String;
    fn start_urls(&self) -> Vec<String>;
    async fn scrape(&self, url: &str) -> Result<Vec<Self::Item>, Self::Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Self::Error>;
}
