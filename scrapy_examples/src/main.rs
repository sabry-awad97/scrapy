use std::time::Duration;

use clap::{Parser, Subcommand};
use error::AppError;
use log::LevelFilter;
use scrapy::Crawler;
use spiders::{BooksSpider, QuotesSpider};

mod error;
mod spiders;

#[derive(Subcommand)]
pub enum Command {
    /// List all spiders
    Spiders,

    /// Run a spider
    Run {
        /// The spider to run
        #[arg(short, long)]
        spider: String,
    },
}

#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Command::Spiders => {
                println!("Listing all spiders...");
                let spider_names = vec!["quotes", "books"];
                for name in spider_names {
                    println!("{}", name);
                }
            }
            Command::Run { spider } => {
                let spider_name = spider.as_str();
                let crawler = Crawler::new(Duration::from_millis(200), 2, 500);

                match spider_name {
                    "quotes" => {
                        let spider = QuotesSpider::new();
                        crawler.crawl(spider).await;
                    }
                    "books" => {
                        let headless = true;
                        let spider = BooksSpider::new(headless).await?;
                        crawler.crawl(spider.clone()).await;
                        spider.close().await?
                    }
                    _ => return Err(AppError::InvalidSpider(spider_name.to_string())),
                };
            }
        }
    }

    Ok(())
}
