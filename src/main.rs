mod parse_args;
mod post;
mod file;
mod export;

use parse_args::{Config, parse_args};
use post::Post;

use anyhow::{Context, Ok, Result};

fn scrape_thread(url: &str, config: &Config) -> Result<()> {
    use std::io::Write;
    let t_total = std::time::Instant::now();

    print!("\tGetting thread...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let html = reqwest::blocking::get(url)
        .with_context(|| format!("HTTP GET failed for {url}"))?
        .text()
        .context("failed to read response body")?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    print!("\tParsing posts...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let posts = Post::parse_posts(&html)
        .context("failed to parse thread HTML")?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    export::export2html(posts, config.files, config.thumb)
        .context("failed to export thread")?;

    println!("Done processing {} ({} ms)", url, t_total.elapsed().as_millis());
    Ok(())
}


fn main() -> Result<()> {
    let config = parse_args()
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

    for url in &config.urls {
        println!("Processing {}:", url);
        scrape_thread(url, &config)
            .unwrap_or_else(|e| eprintln!("Error processing {}: {:#}", url, e));
    }

    Ok(())
}
