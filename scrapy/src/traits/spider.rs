use async_trait::async_trait;

/// An asynchronous trait defining behavior for web spiders, capable of crawling,
/// scraping, and processing content from web pages.
#[async_trait]
pub trait Spider: Send + Sync {
    /// The type of items that the spider extracts from web pages.
    type Item;

    /// The type of error that may occur during spider operations.
    type Error;

    /// Retrieves a human-readable name for the spider.
    ///
    /// # Returns
    ///
    /// A string representing the name of the spider.
    fn name(&self) -> String;

    /// Retrieves the initial URLs for the spider to begin crawling.
    ///
    /// # Returns
    ///
    /// A vector of strings containing the starting URLs.
    fn start_urls(&self) -> Vec<String>;

    /// Asynchronously scrapes content from a given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - A string representing the URL to be scraped.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple with extracted items and new URLs,
    /// or an error describing the scraping failure.
    async fn scrape(&self, url: &str) -> Result<(Vec<Self::Item>, Vec<String>), Self::Error>;

    /// Asynchronously processes an extracted item.
    ///
    /// # Arguments
    ///
    /// * `item` - The item extracted from a web page.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the processing operation.
    async fn process(&self, item: Self::Item) -> Result<(), Self::Error>;
}
