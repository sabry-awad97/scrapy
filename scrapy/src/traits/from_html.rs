pub trait FromHTML {
    type Error;
    type Output;
    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;
}
