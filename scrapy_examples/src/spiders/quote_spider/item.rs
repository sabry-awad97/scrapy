use scraper::{Html, Selector};
use scrapy::FromHTML;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct QuotesItem {
    pub text: Option<String>,
    pub author: Option<String>,
}

impl QuotesItem {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.text.is_none() {
            return Err(AppError::Validation("Missing text".to_string()));
        }

        if self.author.is_none() {
            return Err(AppError::Validation("Missing author".to_string()));
        }

        Ok(())
    }
}

impl FromHTML for QuotesItem {
    type Error = AppError;
    type Output = Vec<Self>;

    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let document = Html::parse_document(html);
        let quote_selector = Selector::parse(".quote").unwrap();
        let author_selector = Selector::parse("small.author").unwrap();
        let text_selector = Selector::parse("span.text").unwrap();

        let mut quotes = Vec::new();

        for quote in document.select(&quote_selector) {
            let text: Option<_> = quote
                .select(&text_selector)
                .next()
                .and_then(|e| e.inner_html().parse().ok());

            let author: Option<_> = quote
                .select(&author_selector)
                .next()
                .and_then(|e| e.inner_html().parse().ok());

            quotes.push(QuotesItem { text, author })
        }

        Ok(quotes)
    }
}
