use anyhow::{anyhow, Result};

/// Downloads a URL, retrying up to `tries` times.
///
/// # Errors
/// Returns an error if all attempts fail or `tries` is 0.
pub fn download(url: &str, tries: u32) -> Result<reqwest::blocking::Response> {
    static CLIENT: std::sync::LazyLock<reqwest::blocking::Client> =
        std::sync::LazyLock::new(reqwest::blocking::Client::new);

    for attempt in 0..tries {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500 * 2u64.pow(attempt)));
        }
        let response = CLIENT.get(url).send()?;
        if response.status().is_success() {
            return Ok(response);
        }
        if response.status().is_client_error() {
            return Err(anyhow!("client error: {}", response.status()));
        }
    }

    Err(anyhow!("failed to download {} after {} tries", url, tries))
}
