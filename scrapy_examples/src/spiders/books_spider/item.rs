use scraper::{Html, Selector};
use scrapy::FromHTML;

use crate::error::AppError;

#[derive(Debug)]
pub struct BookItem {
    pub title: Option<String>,
}

impl FromHTML for BookItem {
    type Error = AppError;
    type Output = Vec<Self>;

    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let document = Html::parse_document(html);
        let book_selector = Selector::parse(".product_pod").unwrap();
        let title_selector = Selector::parse("h3 a").unwrap();

        let mut books = Vec::new();

        for book in document.select(&book_selector) {
            let title: Option<_> = book
                .select(&title_selector)
                .next()
                .and_then(|e| e.inner_html().parse().ok());
            books.push(Self { title })
        }

        Ok(books)
    }
}
