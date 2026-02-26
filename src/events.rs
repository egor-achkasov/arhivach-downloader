#[derive(Debug, Clone)]
pub enum Event {
    // Thread-level lifecycle
    ThreadStarted { url: String, index: usize, total: usize },
    ThreadDone    { url: String, elapsed_ms: u128 },
    ThreadFailed  { url: String, error: String },

    // HTTP fetch
    FetchStarted  { url: String },
    FetchDone     { elapsed_ms: u128 },
    FetchRetrying { url: String, attempt: u32, max_attempts: u32, error: String },

    // HTML parsing
    ParseStarted,
    ParseDone { post_count: usize, elapsed_ms: u128 },

    // Asset downloading
    DownloadBatchStarted  { label: String, total_posts: usize },
    DownloadBatchProgress { label: String, done: usize, total: usize },
    DownloadAssetFailed   { label: String, filename: String, attempt: u32, error: String },
    DownloadAssetSkipped  { label: String, filename: String },
    DownloadBatchDone     { label: String, elapsed_ms: u128 },
}

use std::sync::mpsc;

/// Sink for progress events emitted by the library.
/// Implement this to connect the library to any frontend.
pub trait Reporter: Send + Sync {
    fn report(&self, event: Event);
}

/// Blanket impl: mpsc::Sender<Event> is already a valid Reporter.
impl Reporter for mpsc::Sender<Event> {
    fn report(&self, event: Event) {
        self.send(event).ok();
    }
}

/// No-op reporter — useful in tests or when progress output is not needed.
pub struct NullReporter;

impl Reporter for NullReporter {
    fn report(&self, _event: Event) {}
}
