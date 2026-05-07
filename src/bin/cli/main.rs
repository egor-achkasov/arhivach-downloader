use arhivarch_downloader::config::Config;
use arhivarch_downloader::event::Event;
use arhivarch_downloader::export::{html::HtmlExporter, ExporterKind};

use std::path::PathBuf;
use std::sync::mpsc::channel;

static HELP: &str = "Download threads from arhivach.

Usage: arhivach-downloader-cli.exe [OPTIONS] <URL>

Arguments:
  <URL>  URL to download

Options:
  -d, --dir <DIR>                   Path to download directory [default: .]
  -e, --exporter <EXPORTER>         Exporter [default: html] [possible values: html]
  -t, --thumb                       Download thumbnail images, default: false
  -f, --files                       Download files (images, videos, gifs, etc), default: false
  -r, --resume                      Resume files and thumbnails downloading instead of overwriting. Useless if neither -t nor -f are set, default: false
  -R, --retries <DOWNLOAD_RETRIES>  Download retries in case of a error [default: 3]
  -h, --help                        Print help";

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
    let mut args = std::env::args().skip(1).peekable();

    let mut url: Option<String> = None;
    let mut dir = PathBuf::from(".");
    let mut exporter = ExporterKind::Html(HtmlExporter);
    let mut thumb = false;
    let mut files = false;
    let mut resume = false;
    let mut download_retries: u32 = 3;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            "-t" | "--thumb" => thumb = true,
            "-f" | "--files" => files = true,
            "-r" | "--resume" => resume = true,
            "-d" | "--dir" => {
                let val = args.next().unwrap_or_else(|| {
                    eprintln!("ERROR: {} requires a value", arg);
                    std::process::exit(1);
                });
                dir = PathBuf::from(val);
            }
            "-e" | "--exporter" => {
                let val = args.next().unwrap_or_else(|| {
                    eprintln!("ERROR: {} requires a value", arg);
                    std::process::exit(1);
                });
                exporter = match val.as_str() {
                    "html" => ExporterKind::Html(HtmlExporter),
                    other => {
                        eprintln!("ERROR: unknown exporter '{}'. Possible values: html", other);
                        std::process::exit(1);
                    }
                };
            }
            "-R" | "--retries" => {
                let val = args.next().unwrap_or_else(|| {
                    eprintln!("ERROR: {} requires a value", arg);
                    std::process::exit(1);
                });
                download_retries = val.parse().unwrap_or_else(|_| {
                    eprintln!("ERROR: --retries must be a non-negative integer");
                    std::process::exit(1);
                });
            }
            _ if arg.starts_with('-') => {
                eprintln!("ERROR: unknown option '{}'. Run with --help for usage.", arg);
                std::process::exit(1);
            }
            _ => {
                if url.is_some() {
                    eprintln!("ERROR: unexpected argument '{}'. Run with --help for usage.", arg);
                    std::process::exit(1);
                }
                url = Some(arg);
            }
        }
    }

    let url = url.unwrap_or_else(|| {
        eprintln!("ERROR: missing required argument <URL>. Run with --help for usage.");
        std::process::exit(1);
    });

    Config { url, dir, exporter, thumb, files, resume, download_retries }
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
