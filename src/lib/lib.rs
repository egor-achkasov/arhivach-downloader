pub mod config;
pub mod event;
pub mod export;

mod download;
mod post;

use crate::post::{Post, File};
use crate::export::Exporter;

use anyhow::{Result, Context};

use std::sync::mpsc::Sender;

pub const BASE_URL: &str = "https://arhivach.vc";

pub fn run(config: &config::Config, tx: Sender<event::Event>) -> Result<()> {
    tx.send(event::Event::GetStarted)?;
    let html = download::download(&config.url, config.download_retries)?.text()?;
    let posts = Post::parse_posts(&html)
        .inspect_err(|e| { let _ = tx.send(event::Event::GetFailed { error: format!("{:#}", e) }); })
        .context("failed to parse posts")?;
    tx.send(event::Event::GetDone)?;

    tx.send(event::Event::DownloadAllStarted)?;
    run_download(&posts, &config, tx.clone())
        .inspect_err(|e| { let _ = tx.send(event::Event::DownloadAllFailed { error: format!("{:#}", e) }); })
        .context("failed to download files")?;
    tx.send(event::Event::DownloadAllDone)?;

    tx.send(event::Event::ExportStarted)?;
    config.exporter.export(&posts, config)
        .inspect_err(|e| { let _ = tx.send(event::Event::ExportFailed { error: format!("{:#}", e) }); })
        .context("failed to export")?;
    tx.send(event::Event::ExportDone)?;

    Ok(())
}

/// Download files and thumbnails. Send DownloadStarted, DownloadDone and DownloadFailed events
fn run_download(posts: &[Post], config: &config::Config, tx: Sender<event::Event>) -> Result<()> {
    std::fs::create_dir_all(&config.dir)?;

    let download_item = |url: &str, filepath: &std::path::PathBuf| -> Result<()> {
        let result = download::download(url, config.download_retries)?;
        anyhow::ensure!(result.status().is_success(), "failed to download {}: {}", url, result.status());
        let bytes = result.bytes()?;
        anyhow::ensure!(!bytes.is_empty(), "empty file: {}", url);
        std::fs::write(filepath, bytes)?;
        Ok(())
    };

    let download_section = |
        subdir: &str,
        get_url: fn(&File) -> (&str, &str),
    | -> Result<()> {
        let dir = config.dir.join(subdir);
        std::fs::create_dir_all(&dir)?;

        let mut index: usize = 1;
        let max_index: usize = posts.iter().map(|p| p.files.len()).sum();
        for f in posts.iter().flat_map(|p| &p.files) {
            tx.send(event::Event::DownloadStarted { index, max_index })?;
            let (url, fallback) = get_url(f);
            let filename = url.rsplit("/").next().unwrap_or(fallback).trim();
            let filepath = dir.join(filename);
            if config.resume && filepath.exists() {
                tx.send(event::Event::DownloadSkipped { index, max_index })?;
                index += 1;
                continue
            }
            match download_item(url, &filepath) {
                Ok(()) => tx.send(event::Event::DownloadDone{ index, max_index })?,
                Err(e) => tx.send(event::Event::DownloadFailed {
                    url: url.to_string(),
                    error: format!("{:#}", e)
                })?
            };
            index += 1;
        }
        Ok(())
    };

    if config.files {
        tx.send(event::Event::DownloadFilesStarted)?;
        download_section("files", |f| (&f.url, &f.name_timestamp))?;
        tx.send(event::Event::DownloadFilesDone)?;
    }
    if config.thumb {
        tx.send(event::Event::DownloadThumbStarted)?;
        download_section("thumb", |f| (&f.url_thumb, &f.name_timestamp))?;
        tx.send(event::Event::DownloadThumbDone)?;
    }

    Ok(())
}
