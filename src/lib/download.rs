use anyhow::{anyhow, Result};

/// Downloads a URL, retrying up to `tries` times.
///
/// # Errors
/// Returns an error if all attempts fail or `tries` is 0.
pub fn download(url: &str, tries: u32) -> Result<ureq::http::Response<ureq::Body>> {
    static CLIENT: std::sync::LazyLock<ureq::Agent> =
        std::sync::LazyLock::new(ureq::Agent::new_with_defaults);
    for attempt in 0..tries {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500 * 2u64.pow(attempt)));
        }
        match CLIENT.get(url).call() {
            Ok(response) => return Ok(response),
            Err(ureq::Error::StatusCode(code)) if code >= 400 && code < 500 => {
                return Err(anyhow!("client error: {}", code));
            }
            Err(ureq::Error::StatusCode(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(anyhow!("failed to download {} after {} tries", url, tries))
}
