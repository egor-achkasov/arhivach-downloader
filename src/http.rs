use anyhow::{Context, Result};

use crate::events::{Event, Reporter};

/// GET a URL with up to `attempts` retries, reporting each failure via `reporter`.
pub fn fetch_with_retry(url: &str, attempts: u32, reporter: &dyn Reporter) -> Result<String> {
    for attempt in 1..=attempts {
        match reqwest::blocking::get(url).and_then(|r| r.text()) {
            Ok(text) => return Ok(text),
            Err(e) => {
                reporter.report(Event::FetchRetrying {
                    url: url.to_string(),
                    attempt,
                    max_attempts: attempts,
                    error: e.to_string(),
                });
                if attempt < attempts {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                }
            }
        }
    }
    anyhow::bail!("failed to get thread after {attempts} attempts")
}

/// Download a single URL and write it to `path`.
pub fn download(url: &str, path: &str) -> Result<()> {
    let bytes = reqwest::blocking::get(url)
        .with_context(|| format!("HTTP GET failed for {}", url))?
        .bytes()
        .context("failed to read response body")?;
    std::fs::write(path, &bytes)
        .with_context(|| format!("failed to write {}", path))?;
    Ok(())
}
