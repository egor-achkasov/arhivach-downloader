use arhivarch_downloader::config::Config;
use arhivarch_downloader::event::Event;
use arhivarch_downloader::export::{html::HtmlExporter, ExporterKind};

use clap::{Parser, ValueEnum};

use std::path::PathBuf;

#[derive(Clone, ValueEnum)]
enum ExporterArg {
    Html,
}
use std::sync::mpsc::channel;

fn main() {
    let config = parse_args();
    let (tx, rx) = channel::<Event>();
    let handle = std::thread::spawn({
        let config = config.clone();
        move || arhivarch_downloader::run(&config, tx)
    });

    for event in rx {
        render_event(&event);
    }

    handle.join().map_err(|e| {
        eprintln!("ERROR: {:?}", e);
        std::process::exit(1);
    }).ok();
}

pub fn parse_args() -> Config {
    #[derive(Parser)]
    #[command(about, long_about)]
    struct Cli {
        /// URL to download
        url: String,

        /// Path to download directory
        #[arg(short = 'd', long = "dir", value_name = "DIR", default_value = ".", value_hint = clap::ValueHint::DirPath)]
        dir: PathBuf,

        /// Exporter
        #[arg(short = 'e', long = "exporter", value_name = "EXPORTER", default_value = "html")]
        exporter: ExporterArg,

        /// Download thumbnail images, default: false
        #[arg(short = 't', long = "thumb", default_value_t = false)]
        thumb: bool,

        /// Download files (images, videos, gifs, etc), default: false
        #[arg(short = 'f', long = "files", default_value_t = false)]
        files: bool,

        /// Resume files and thumbnails downloading instead of overwriting. Useless if neither -t nor -f are set, default: false
        #[arg(short = 'r', long = "resume", default_value_t = false)]
        resume: bool,

        /// Download retries in case of a error
        #[arg(short = 'R', long = "retries", default_value_t = 3)]
        download_retries: u32,
    }
    let cli = Cli::parse();

    Config {
        url: cli.url,
        dir: cli.dir,
        exporter: match cli.exporter {
            ExporterArg::Html => ExporterKind::Html(HtmlExporter),
        },
        thumb: cli.thumb,
        files: cli.files,
        resume: cli.resume,
        download_retries: cli.download_retries,
    }
}

fn render_event(event: &Event) {
    use std::io::Write;
    match event {
        Event::GetStarted => {
            print!("Fetching thread...");
            std::io::stdout().flush().ok();
        }
        Event::GetDone =>
            println!(" Done."),
        Event::GetFailed { error } =>
            eprintln!("\nFailed to fetch thread: {}", error),

        Event::DownloadAllStarted =>
            println!("Downloading stuff..."),
        Event::DownloadAllDone =>
            println!("All downloads complete."),
        Event::DownloadAllFailed { error } =>
            eprintln!("Download failed: {}", error),

        Event::DownloadStarted { index, max_index } => {
            print!("\r\tDownloading {} / {}...", index, max_index);
            std::io::stdout().flush().ok();
        }
        Event::DownloadDone { index, max_index } => {
            println!("\r\tDownloading {} / {}... Done.", index, max_index);
        }
        Event::DownloadFailed { url, error } =>
            eprintln!("\r\tFailed to download {}: {}", url, error),
        Event::DownloadSkipped { index, max_index } =>
            println!("\r\tDownloading {} / {}... Skipped.", index, max_index),

        Event::DownloadFilesStarted => {
            println!("Downloading files...");
            std::io::stdout().flush().ok();
        }
        Event::DownloadFilesDone =>
            println!("Done."),
        Event::DownloadThumbStarted => {
            println!("Downloading thumbnails...");
            std::io::stdout().flush().ok();
        }
        Event::DownloadThumbDone =>
            println!("Done."),

        Event::ExportStarted => {
            print!("Exporting...");
            std::io::stdout().flush().ok();
        }
        Event::ExportDone =>
            println!(" Done."),
        Event::ExportFailed { error } =>
            eprintln!("\nExport failed: {}", error),
    }
}
