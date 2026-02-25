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
