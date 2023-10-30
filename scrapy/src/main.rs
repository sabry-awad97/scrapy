use reqwest::Client;

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let url = "https://quotes.toscrape.com/page/1/";
    let http_client = Client::builder().build().expect("Building HTTP client");

    log::info!("visiting: {}", url);
    let http_res = http_client.get(url).send().await?.text().await?;

    println!("{http_res}");

    Ok(())
}
