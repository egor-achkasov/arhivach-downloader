use crate::{config::Config, post::Post};
use anyhow::Result;
use super::Exporter;

mod render;

const TEMPLATE: &str = include_str!("template.html");

#[derive(Clone)]
pub struct HtmlExporter;

impl Exporter for HtmlExporter {
    fn export(&self, posts: &[Post], config: &Config) -> Result<()> {
        anyhow::ensure!(!posts.is_empty(), "No posts to export");

        std::fs::create_dir_all(&config.dir)?;
        let posts_html = posts
            .iter()
            .map(|p| render::render_post(p, config.files, config.thumb))
            .collect::<Vec<String>>()
            .join("\n");
        let index_html = TEMPLATE.replace("{{posts}}", &posts_html);
        std::fs::write(config.dir.join("index.html"), index_html)?;

        Ok(())
    }
}
