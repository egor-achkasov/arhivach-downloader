mod parse_args;
mod post;
mod file;
mod export;

use parse_args::{Config, parse_args};
use post::Post;

use anyhow::{Context, Ok, Result};

async fn scrape_thread(url: &str, config: &Config) -> Result<()> {
    let html = reqwest::get(url).await
        .with_context(|| format!("HTTP GET failed for {url}"))?
        .text().await
        .context("failed to read response body")?;
    let posts = Post::parse_posts(&html)
        .context("failed to parse thread HTML")?;
    export::export2html(posts, config.files, config.thumb).await
        .context("failed to export thread")?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()>{
    let config = parse_args()
        .unwrap_or_else(|e| {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        });

    for (i, url) in config.urls.iter().enumerate() {
        println!("Processing: {} ({} / {})", url, i + 1, config.urls.len());
        scrape_thread(url, &config).await
            .unwrap_or_else(|e| eprintln!("Error processing {}: {}", url, e));
    }

    println!("Done");
    Ok(())
}
