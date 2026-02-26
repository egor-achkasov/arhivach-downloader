use crate::{config::Config, events::{Event, Reporter}, http, post::{File, Post}, render};

use anyhow::{Result, Context};

const TEMPLATE: &str = include_str!("../template.html");

/// Write a top-level index.html with one entry per thread (first post + link to thread folder)
pub fn write_index_html(first_posts: &[Post], config: &Config) -> Result<()> {
    if first_posts.is_empty() {
        return Ok(());
    }

    let posts_html: String = first_posts
        .iter()
        .map(|p| {
            let mut post_html = render::render_post(p, config.files, config.thumb);
            // render_post references thumbnails and images in the same directory,
            // so replace them with links to the thread folder
            config.files.then(|| post_html = post_html.replace(
                "<a href=\"files/",
                &format!("<a href=\"{}/files/", p.id),
            ));
            config.thumb.then(|| post_html = post_html.replace(
                "<img src=\"thumb/",
                &format!("<img src=\"{}/thumb/", p.id),
            ));
            format!("<div><a href=\"{}/index.html\">В тред &rarr;</a></div>{}\n", p.id, post_html)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let index_html = TEMPLATE.replace("{{posts}}", &posts_html);
    std::fs::write("index.html", index_html)
        .context("failed to write index.html")?;

    Ok(())
}

/// Export the thread to a simple static HTML
///
/// Creates a directory as follows:
/// ./{thread_id}, where {thread_id} is OP ID
/// If download_files is true, downloads files to ./{thread_id}/files
/// If download_thumbnails is true, downloads thumbnails to ./{thread_id}/thumb
///
/// WARNING: If the directory already exists, it will be overwritten
pub fn export2html(posts: &[Post], config: &Config, reporter: &dyn Reporter) -> Result<()> {
    if posts.is_empty() {
        anyhow::bail!("No posts to export");
    }

    let dir = format!("{}", posts[0].id);
    std::fs::create_dir_all(&dir)?;

    let posts_html: String = posts
        .iter()
        .map(|p| render::render_post(p, config.files, config.thumb))
        .collect::<Vec<String>>()
        .join("\n");

    if config.files {
        download_assets(
            &posts,
            &format!("{}/files", dir),
            "files",
            |f| &f.url,
            config.resume,
            reporter,
        )?;
    }
    if config.thumb {
        download_assets(
            &posts,
            &format!("{}/thumb", dir),
            "thumbnails",
            |f| &f.url_thumb,
            config.resume,
            reporter,
        )?;
    }

    let index_html = TEMPLATE.replace("{{posts}}", &posts_html);
    std::fs::write(format!("{}/index.html", dir), index_html)?;

    Ok(())
}

fn download_assets(
    posts: &[Post],
    dest_dir: &str,
    label: &str,
    url_of: impl Fn(&File) -> &str,
    skip_if_exists: bool,
    reporter: &dyn Reporter,
) -> Result<()> {
    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory {}", dest_dir))?;

    let t = std::time::Instant::now();
    reporter.report(Event::DownloadBatchStarted {
        label: label.to_string(),
        total_posts: posts.len(),
    });

    for (i, post) in posts.iter().enumerate() {
        for f in &post.files {
            let url = url_of(f);
            let filename = url.split('/').last().unwrap_or("").to_string();
            let path = format!("{}/{}", dest_dir, filename);
            if skip_if_exists && std::path::Path::new(&path).exists() {
                continue;
            }
            let mut result = Err(anyhow::anyhow!("no attempts"));
            for attempt in 1..=3u32 {
                result = http::download(url, &path);
                if result.is_ok() { break; }
                let e = result.as_ref().unwrap_err();
                reporter.report(Event::DownloadAssetFailed {
                    label: label.to_string(),
                    filename: filename.clone(),
                    attempt,
                    error: e.to_string(),
                });
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if result.is_err() {
                reporter.report(Event::DownloadAssetSkipped {
                    label: label.to_string(),
                    filename: filename.clone(),
                });
            }
        }
        reporter.report(Event::DownloadBatchProgress {
            label: label.to_string(),
            done: i + 1,
            total: posts.len(),
        });
    }

    reporter.report(Event::DownloadBatchDone {
        label: label.to_string(),
        elapsed_ms: t.elapsed().as_millis(),
    });

    Ok(())
}
