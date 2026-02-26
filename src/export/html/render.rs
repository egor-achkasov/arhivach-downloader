use crate::thread::{File, Post};

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
pub fn render_text_to_html(text: &str) -> String {
    let needle = "&gt;&gt;";

    let lines: Vec<String> = text.split('\n').map(|line| {
        let escaped = html_escape(line);

        // Replace >>id with reply link anchors
        let mut processed = String::with_capacity(escaped.len());
        let mut rest = escaped.as_str();
        while let Some(pos) = rest.find(needle) {
            processed.push_str(&rest[..pos]);
            let after = &rest[pos + needle.len()..];
            let digit_end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
            if digit_end > 0 {
                let id = &after[..digit_end];
                processed.push_str(&format!("<a href=\"#post{id}\" class=\"reply-link\">&gt;&gt;{id}</a>"));
                rest = &after[digit_end..];
            } else {
                processed.push_str(needle);
                rest = after;
            }
        }
        processed.push_str(rest);

        // Wrap in greentext span if line starts with > but not >>digit
        let is_greentext = escaped.starts_with("&gt;")
            && !escaped.strip_prefix(needle).is_some_and(|s| s.starts_with(|c: char| c.is_ascii_digit()));
        if is_greentext {
            format!("<span class=\"quote\">{processed}</span>")
        } else {
            processed
        }
    }).collect();

    lines.join("<br>\n")
}

/// Renders a single post to an HTML fragment string.
pub fn render_post(post: &Post, download_files: bool, download_thumbnails: bool) -> String {
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
    files: &[File],
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
