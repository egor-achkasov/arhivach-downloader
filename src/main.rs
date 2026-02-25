mod cli;

use arhivarch_downloader::{backend, events::Event};
use std::sync::mpsc;

fn main() -> anyhow::Result<()> {
    let config = cli::parse_args().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let (tx, rx) = mpsc::channel::<Event>();

    let handle = std::thread::spawn({
        let config = config.clone();
        move || backend::run(&config, tx)
    });

    for event in rx {
        render_event(&event);
    }

    handle.join().unwrap()
}

fn render_event(event: &Event) {
    use std::io::Write;
    match event {
        Event::ThreadStarted { url, index, total } =>
            println!("Processing {} ({} / {}):", url, index, total),

        Event::ThreadDone { url, elapsed_ms } =>
            println!("Done processing {} ({} ms)", url, elapsed_ms),

        Event::ThreadFailed { url, error } =>
            eprintln!("Error processing {}: {}", url, error),

        Event::FetchStarted { .. } => {
            print!("\tGetting thread...");
            std::io::stdout().flush().ok();
        }

        Event::FetchDone { elapsed_ms } =>
            println!(" Done ({} ms)", elapsed_ms),

        Event::FetchRetrying { url, attempt, max_attempts, error } => {
            eprintln!("\n\tHTTP request failed for {}: {}", url, error);
            if attempt < max_attempts {
                eprintln!("\tWaiting 3 seconds...");
            }
        }

        Event::ParseStarted => {
            print!("\tParsing posts...");
            std::io::stdout().flush().ok();
        }

        Event::ParseDone { elapsed_ms, .. } =>
            println!(" Done ({} ms)", elapsed_ms),

        Event::DownloadBatchStarted { label, total_posts } => {
            print!("\tDownloading {}... post 0 / {}", label, total_posts);
            std::io::stdout().flush().ok();
        }

        Event::DownloadBatchProgress { label, done, total } => {
            print!("\r\tDownloading {}... post {} / {}", label, done, total);
            std::io::stdout().flush().ok();
        }

        Event::DownloadAssetFailed { label, filename, error, .. } =>
            println!("\r\tFailed to download {} {}: {}\n\t-> Waiting 3 seconds...", label, filename, error),

        Event::DownloadAssetSkipped { label, filename } =>
            println!("\tSkipping {} {} after 3 failed attempts.", label, filename),

        Event::DownloadBatchDone { elapsed_ms, .. } =>
            println!(" Done ({} ms)", elapsed_ms),
    }
}
