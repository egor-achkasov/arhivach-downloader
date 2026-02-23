use crate::{parse_args::Config, post::Post};

use anyhow::{Result, Context};

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

/// Converts plain post text to HTML.
/// - `>>id` → reply link anchor
/// - Lines starting with `>` (not `>>digit`) → greentext span
/// - `\n` → `<br>`
fn render_text_to_html(text: &str) -> String {
    static RE_REPLY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"&gt;&gt;(\d+)").unwrap()
    });

    let lines: Vec<String> = text.split('\n').map(|line| {
        let escaped = html_escape(line);
        // Greentext: starts with > but not >>digit
        let processed = if escaped.starts_with("&gt;") && !escaped.starts_with("&gt;&gt;") {
            format!("<span class=\"quote\">{}</span>", escaped)
        } else {
            escaped
        };
        // Reply links: >>id
        RE_REPLY.replace_all(&processed, |caps: &regex::Captures| {
            let id = &caps[1];
            format!("<a href=\"#post{}\" class=\"reply-link\">&gt;&gt;{}</a>", id, id)
        }).into_owned()
    }).collect();

    lines.join("<br>\n")
}

/// Export the thread to a simple static HTML
///
/// Creates a directory as follows:
/// ./{thread_id}, where {thread_id} is OP ID
/// If download_files is true, downloads files to ./{thread_id}/files
/// If download_thumbnails is true, downloads thumbnails to ./{thread_id}/thumb
///
/// WARNING: If the directory already exists, it will be overwritten
pub fn export2html(posts: &[Post], config: &Config) -> Result<()> {
    if posts.is_empty() {
        anyhow::bail!("No posts to export");
    }

    let dir = format!("{}", posts[0].id);
    std::fs::create_dir_all(&dir)?;

    let posts_html: String = posts
        .iter()
        .map(|p| render_post(p, config.files, config.thumb))
        .collect::<Vec<String>>()
        .join("\n");

    if config.files {
        download_assets(
            &posts,
            &format!("{}/files", dir),
            "files",
            |f| &f.url,
            config.resume,
        )?;
    }
    if config.thumb {
        download_assets(
            &posts,
            &format!("{}/thumb", dir),
            "thumbnails",
            |f| &f.url_thumb,
            config.resume,
        )?;
    }

    const TEMPLATE: &'static str = include_str!("../template.html");
    let index_html = TEMPLATE.replace("{{posts}}", &posts_html);
    std::fs::write(format!("{}/index.html", dir), index_html)?;

    Ok(())
}

fn render_post(post: &Post, download_files: bool, download_thumbnails: bool) -> String {
    let mut html = format!("<div class=\"post\" id=\"post{}\">\n", post.id);

    html.push_str("  <div class=\"post-head\">\n");

    // Subject
    if let Some(ref subject) = post.subject {
        html.push_str(&format!(
            "    <span class=\"post-subject\">{}</span>\n",
            html_escape(subject)
        ));
    }

    // Name /w mailto/sage
    let name = post.name.as_deref().unwrap_or("Аноним");
    let name_display = if let Some(ref mailto) = post.mailto {
        format!("[{}] {}", mailto, name)
    } else {
        name.to_string()
    };
    html.push_str(&format!(
        "    <span class=\"post-name\">{}</span>\n",
        html_escape(&name_display)
    ));

    // Time, num, id
    html.push_str(&format!("    <span class=\"post-time\">{}</span>\n", html_escape(&post.time)));
    html.push_str(&format!("    <span class=\"post-num\">{}</span>\n", html_escape(&post.num)));
    html.push_str(&format!(
        "    <span class=\"post-id\"><a href=\"#post{0}\">№{0}</a></span>\n",
        post.id
    ));

    html.push_str("  </div>\n");

    // Images
    html.push_str(&render_images(&post.files, download_files, download_thumbnails));

    // Body
    html.push_str("  <div class=\"post-body\">\n");
    if !post.text.is_empty() {
        html.push_str("    ");
        html.push_str(&render_text_to_html(&post.text));
        html.push('\n');
    }
    html.push_str("  </div>\n");

    html.push_str("</div>\n");
    html
}

fn render_images(
    files: &[crate::file::File],
    download_files: bool,
    download_thumbnails: bool,
) -> String {
    if files.is_empty() {
        return String::new();
    }

    let mut html = String::from("  <div class=\"post-images\">\n");
    for file in files {
        let href = if download_files && !file.url.is_empty() {
            format!("files/{}", file.url.split('/').last().unwrap_or(""))
        } else {
            file.url.clone()
        };

        let thumb_filename = file.url_thumb.split('/').last().unwrap_or("").to_string();
        let img_src = if download_thumbnails && !file.url_thumb.is_empty() {
            format!("thumb/{}", thumb_filename)
        } else {
            file.url_thumb.clone()
        };

        html.push_str(&format!(
            "    <div class=\"post-image\">\n      <a href=\"{}\" target=\"_blank\" title=\"{}\">\n        <img src=\"{}\" alt=\"\" loading=\"lazy\">\n      </a>\n      <div class=\"post-image-info\">{} (<a href=\"{}\" target=\"_blank\" class=\"post-image-link\">o</a>, <a href=\"{}\" target=\"_blank\" class=\"post-image-link\">t</a>)</div>\n    </div>\n",
            html_escape(&href),
            html_escape(&file.name_orig),
            html_escape(&img_src),
            html_escape(&file.name_orig),
            html_escape(&file.url),
            html_escape(&file.url_thumb),
        ));
    }
    html.push_str("  </div>\n");
    html
}


fn download_assets(
    posts: &[Post],
    dest_dir: &str,
    label: &str,
    url_of: impl Fn(&crate::file::File) -> &str,
    skip_if_exists: bool,
) -> Result<()> {
    use std::io::Write;

    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory {}", dest_dir))?;
    let t = std::time::Instant::now();
    print!("\tDownloading {}... post 0 / {}", label, posts.len());
    std::io::stdout().flush().ok();
    for (i, post) in posts.iter().enumerate() {
        for f in &post.files {
            let url = url_of(f);
            let filename = url.split('/').last().unwrap_or("");
            let path = format!("{}/{}", dest_dir, filename);
            if skip_if_exists && std::path::Path::new(&path).exists() {
                continue;
            }
            let mut result = Err(anyhow::anyhow!("no attempts"));
            for _ in 0..3 {
                result = download(url, &path);
                if result.is_ok() { break; }
                let e = result.as_ref().unwrap_err();
                println!("\r\tFailed to download {} {}: {}\n\t-> Waiting 3 seconds...", label, filename, e);
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if result.is_err() {
                println!("\tSkipping {} {} after 3 failed attempts.", label, filename);
            }
        }
        print!("\r\tDownloading {}... post {} / {}", label, i + 1, posts.len());
        std::io::stdout().flush().ok();
    }
    println!(" Done ({} ms)", t.elapsed().as_millis());
    Ok(())
}

fn download(url: &str, path: &str) -> Result<()> {
    let bytes = reqwest::blocking::get(url)
        .with_context(|| format!("HTTP GET failed for {}", url))?
        .bytes()
        .context("failed to read response body")?;
    std::fs::write(path, &bytes)
        .with_context(|| format!("failed to write {}", path))?;
    Ok(())
}
