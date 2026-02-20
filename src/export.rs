use crate::post::Post;

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
/// ./{thread_name}, where {thread_name} is OP subject or first 20 characters of OP text
/// If download_files is true, downloads files to ./{thread_name}/files
/// If download_thumbnails is true, downloads thumbnails to ./{thread_name}/thumb
///
/// WARNING: If the directory already exists, it will be overwritten
pub async fn export2html(
    posts: Vec<Post>,
    download_files: bool,
    download_thumbnails: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if posts.is_empty() {
        return Err("No posts to export".into());
    }

    // Get thread name
    let thread_name: String = format!(
        "{}_{}",
        posts[0].subject.clone().unwrap_or_else(|| posts[0].text.chars().take(20).collect()),
        posts[0].id
    );

    // Create directories
    let dir = format!("{}", thread_name);
    std::fs::create_dir_all(&dir)?;

    // Render the thread
    let posts_html: String = posts
        .iter()
        .map(|p| render_post(p, download_files, download_thumbnails))
        .collect::<Vec<String>>()
        .join("\n");
    // Download files
    if download_files {
        let dir = format!("{}/files", dir);
        std::fs::create_dir_all(&dir)?;
        for (f, filename) in posts.iter().flat_map(|p| &p.files)
            .filter_map(|f| f.url.split('/').last().map(|name| (f, name)))
        {
            let path = format!("{}/{}", dir, filename);
            download(&f.url, &path).await?;
        }
    }
    // Download thumbnails
    if download_thumbnails {
        let dir = format!("{}/thumb", dir);
        std::fs::create_dir_all(&dir)?;
        for (f, filename) in posts.iter().flat_map(|p| &p.files)
            .filter_map(|f| f.url_thumb.split('/').last().map(|name| (f, name)))
        {
            let path = format!("{}/{}", dir, filename);
            download(&f.url_thumb, &path).await?;
        }
    }

    // Insert the posts html into a template and write as index.html
    let template = std::fs::read_to_string("template.html")?
        .replace("{{posts}}", &posts_html);
    std::fs::write(format!("{}/index.html", dir), template)?;
    
    Ok(())
}

fn render_post(
    post: &Post,
    download_files: bool,
    download_thumbnails: bool,
) -> String {
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
            format!("files/{}", file.name_timestamp)
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


async fn download(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    std::fs::write(path, &bytes)?;
    Ok(())
}
