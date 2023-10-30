/// A trait for types that can be constructed from HTML content.
pub trait FromHTML {
    /// The type of error that may occur during construction.
    type Error;

    /// The resulting type after successful construction.
    type Output;

    /// Attempts to construct an instance from the provided HTML content.
    ///
    /// # Arguments
    ///
    /// * `html` - A string containing the HTML content to be parsed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the constructed instance on success,
    /// or an error describing the parsing failure.
    ///
    fn from_html(html: &str) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;
}
