#[derive(Debug, Clone)]
pub enum Event {
    // Thread retrieval
    GetStarted,
    GetDone,
    GetFailed { error: String },

    // Files download
    DownloadAllStarted,
    DownloadAllDone,
    DownloadAllFailed { error: String },

    // File download
    DownloadStarted { index: usize, max_index: usize },
    DownloadDone { index: usize, max_index: usize },
    DownloadSkipped { index: usize, max_index: usize },
    DownloadFailed { url: String, error: String },    
    
    // Files and thumbnails download
    DownloadFilesStarted,
    DownloadFilesDone,
    DownloadThumbStarted,
    DownloadThumbDone,

    // Thread export
    ExportStarted,
    ExportDone,
    ExportFailed { error: String },
}
