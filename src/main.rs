mod parse_args;
mod post;
mod file;

use parse_args::{Config, parse_args};
use post::Post;

use std::fs;

const ARHIVACH_DOMAIN_NAME: &str = "arhivach.vc";

async fn download_html(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    Ok(html)
}

/// Validate and sanitize arhivach thread URL
/// param url: URL to validate (https?://arhivach\.vc/thread/\d{7}/?)
/// Returns None if the URL is invalid
/// Returns Some(thread_number) if the URL is valid
fn validate_and_sanitize_url(url: &str) -> Option<u32> {
    let url = url.trim().trim_end_matches('/');
    let parts: Vec<&str> = url.split('/').collect();

    // Expect: ["https:" or "http:", "", "arhivach.vc", "thread", "<number>"]
    if parts.len() != 5 
    || parts[0] != "https:" && parts[0] != "http:" 
    || parts[2] != ARHIVACH_DOMAIN_NAME 
    || parts[3] != "thread" {
        return None;
    }

    parts[4].parse::<u32>().ok()
}

async fn scrape_thread(url: &str, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let thread_number = validate_and_sanitize_url(url)
        .ok_or_else(|| format!("invalid URL: {}", url))?;
    let html = download_html(url).await?;

    let dir = thread_number.to_string();
    fs::create_dir_all(&dir)?;

    // Get posts
    let posts = Post::parse_posts(&html)?;

    // DELETE
    for post in posts {
        println!("{}", post);
        // wait for user to press any button
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    for (i, url) in config.urls.iter().enumerate() {
        println!("Processing: {} ({} / {})", url, i + 1, config.urls.len());
        if let Err(e) = scrape_thread(url, &config).await {
            eprintln!("Error processing {}: {}", url, e);
        }
    }

    println!("Done");
}
