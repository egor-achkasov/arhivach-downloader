mod parse_args;
mod post;
mod file;
mod export;

use parse_args::{Config, parse_args};
use post::Post;

use anyhow::{Context, Ok, Result};
use std::result::Result::Ok as StdOk;

fn fetch_with_retry(url: &str, attempts: u32) -> Result<String> {
    for attempt in 1..=attempts {
        match reqwest::blocking::get(url).and_then(|r| r.text()) {
            StdOk(text) => return Ok(text),
            Err(e) => {
                eprintln!("\n\tHTTP request failed for {url}: {e}");
                if attempt < attempts {
                    eprintln!("\tWaiting 3 seconds...");
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }
        }
    }
    anyhow::bail!("failed to get thread after {attempts} attempts")
}

fn scrape_thread(url: &str, config: &Config) -> Result<Post> {
    use std::io::Write;
    let t_total = std::time::Instant::now();

    print!("\tGetting thread...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let html = fetch_with_retry(url, 3)?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    print!("\tParsing posts...");
    std::io::stdout().flush().ok();
    let t = std::time::Instant::now();
    let posts = Post::parse_posts(&html)
        .context("failed to parse thread HTML")?;
    println!(" Done ({} ms)", t.elapsed().as_millis());

    let first_post = posts.first().context("thread has no posts")?.clone();

    export::export2html(&posts, &config)
        .context("failed to export thread")?;

    println!("Done processing {} ({} ms)", url, t_total.elapsed().as_millis());
    Ok(first_post)
}


fn main() -> Result<()> {
    let config = parse_args()
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

    let mut first_posts: Vec<Post> = Vec::new();
    let mut i = 1;
    for url in &config.urls {
        println!("Processing {} ({} / {}):", url, i, config.urls.len());
        i += 1;
        match scrape_thread(url, &config) {
            StdOk(first_post) => first_posts.push(first_post),
            Err(e) => eprintln!("Error processing {}: {:#}", url, e),
        }
    }

    export::write_index_html(&first_posts, &config)
        .context("failed to write main index.html")?;

    Ok(())
}
