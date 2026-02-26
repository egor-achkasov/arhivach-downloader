use anyhow::{Context, Ok, Result};
use std::result::Result::Ok as StdOk;

use crate::{config::Config, events::{Event, Reporter}, export, http, post::Post};

pub fn scrape_thread(url: &str, config: &Config, reporter: &dyn Reporter) -> Result<Post> {
    let t_total = std::time::Instant::now();

    reporter.report(Event::FetchStarted { url: url.to_string() });
    let t = std::time::Instant::now();
    let html = http::fetch_with_retry(url, 3, reporter)?;
    reporter.report(Event::FetchDone { elapsed_ms: t.elapsed().as_millis() });

    reporter.report(Event::ParseStarted);
    let t = std::time::Instant::now();
    let posts = Post::parse_posts(&html).context("failed to parse thread HTML")?;
    reporter.report(Event::ParseDone {
        post_count: posts.len(),
        elapsed_ms: t.elapsed().as_millis(),
    });

    let first_post = posts.first().context("thread has no posts")?.clone();

    export::export2html(&posts, config, reporter).context("failed to export thread")?;

    reporter.report(Event::ThreadDone {
        url: url.to_string(),
        elapsed_ms: t_total.elapsed().as_millis(),
    });

    Ok(first_post)
}

pub fn run(config: &Config, reporter: &dyn Reporter) -> Result<()> {
    let total = config.urls.len();
    let mut first_posts: Vec<Post> = Vec::new();

    for (i, url) in config.urls.iter().enumerate() {
        reporter.report(Event::ThreadStarted {
            url: url.clone(),
            index: i + 1,
            total,
        });

        match scrape_thread(url, config, reporter) {
            StdOk(first_post) => first_posts.push(first_post),
            Err(e) => {
                reporter.report(Event::ThreadFailed {
                    url: url.clone(),
                    error: format!("{:#}", e),
                });
            }
        }
    }

    export::write_index_html(&first_posts, config).context("failed to write main index.html")?;

    Ok(())
}
