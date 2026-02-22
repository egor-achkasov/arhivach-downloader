mod parse_args;
mod post;
mod file;
mod export;

use parse_args::{Config, parse_args};
use post::Post;

use anyhow::{Context, Ok, Result};

async fn scrape_thread(url: &str, config: &Config) -> Result<()> {
    use std::io::Write;
    let t_total = std::time::Instant::now();

    print!("\tGetting thread...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let html = reqwest::get(url).await
        .with_context(|| format!("HTTP GET failed for {url}"))?
        .text().await
        .context("failed to read response body")?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    print!("\tParsing posts...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let posts = Post::parse_posts(&html)
        .context("failed to parse thread HTML")?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    export::export2html(posts, config.files, config.thumb).await
        .context("failed to export thread")?;

    println!("Done processing {} ({} ms)", url, t_total.elapsed().as_millis());
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"))
        )
        .init();

    let config = parse_args()
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

    for url in &config.urls {
        println!("Processing {}:", url);
        scrape_thread(url, &config).await
            .unwrap_or_else(|e| eprintln!("Error processing {}: {:#}", url, e));
    }

    Ok(())
}
