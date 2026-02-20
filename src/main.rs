mod parse_args;
mod post;
mod file;
mod export;

use parse_args::{Config, parse_args};
use post::Post;

async fn scrape_thread(url: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Validate URL (expect https?://arhivach\.vc/thread/\d{7}/?)
    let is_valid = matches!(
        url.trim().trim_end_matches('/').split('/').collect::<Vec<_>>().as_slice(),
        ["https:" | "http:", "", "arhivach.vc", "thread", _]
    );
    if !is_valid {
        return Err("invalid URL".into());
    }

    let html = reqwest::get(url).await?.text().await?;
    let posts = Post::parse_posts(&html)?;
    export::export2html(posts, config.files, config.thumb).await?;

    Ok(())
}


#[tokio::main]
async fn main() {
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
}
