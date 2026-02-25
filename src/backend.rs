use std::sync::mpsc::Sender;

use anyhow::{Context, Ok, Result};
use std::result::Result::Ok as StdOk;

use crate::{config::Config, events::Event, export, post::Post};

pub fn fetch_with_retry(url: &str, attempts: u32, tx: &Sender<Event>) -> Result<String> {
    for attempt in 1..=attempts {
        match reqwest::blocking::get(url).and_then(|r| r.text()) {
            StdOk(text) => return Ok(text),
            Err(e) => {
                tx.send(Event::FetchRetrying {
                    url: url.to_string(),
                    attempt,
                    max_attempts: attempts,
                    error: e.to_string(),
                }).ok();
                if attempt < attempts {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }
        }
    }
    anyhow::bail!("failed to get thread after {attempts} attempts")
}

pub fn scrape_thread(url: &str, config: &Config, tx: &Sender<Event>) -> Result<Post> {
    let t_total = std::time::Instant::now();

    tx.send(Event::FetchStarted { url: url.to_string() }).ok();
    let t = std::time::Instant::now();
    let html = fetch_with_retry(url, 3, tx)?;
    tx.send(Event::FetchDone { elapsed_ms: t.elapsed().as_millis() }).ok();

    tx.send(Event::ParseStarted).ok();
    let t = std::time::Instant::now();
    let posts = Post::parse_posts(&html).context("failed to parse thread HTML")?;
    tx.send(Event::ParseDone {
        post_count: posts.len(),
        elapsed_ms: t.elapsed().as_millis(),
    }).ok();

    let first_post = posts.first().context("thread has no posts")?.clone();

    export::export2html(&posts, config, tx).context("failed to export thread")?;

    tx.send(Event::ThreadDone {
        url: url.to_string(),
        elapsed_ms: t_total.elapsed().as_millis(),
    }).ok();

    Ok(first_post)
}

pub fn run(config: &Config, tx: Sender<Event>) -> Result<()> {
    let total = config.urls.len();
    let mut first_posts: Vec<Post> = Vec::new();

    for (i, url) in config.urls.iter().enumerate() {
        tx.send(Event::ThreadStarted {
            url: url.clone(),
            index: i + 1,
            total,
        }).ok();

        match scrape_thread(url, config, &tx) {
            StdOk(first_post) => first_posts.push(first_post),
            Err(e) => {
                tx.send(Event::ThreadFailed {
                    url: url.clone(),
                    error: format!("{:#}", e),
                }).ok();
            }
        }
    }

    export::write_index_html(&first_posts, config).context("failed to write main index.html")?;

    Ok(())
}
