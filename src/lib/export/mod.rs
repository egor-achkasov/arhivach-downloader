pub mod html;

use super::{config::Config, post::Post};
use crate::error::{Error, Result};

use std::str::FromStr;

#[derive(Clone)]
pub enum ExporterKind {
    Html(html::HtmlExporter),
}

pub trait Exporter {
    fn export(&self, posts: &[Post], config: &Config) -> Result<()>;
}

impl Exporter for ExporterKind {
    fn export(&self, posts: &[Post], config: &Config) -> Result<()> {
        match self {
            ExporterKind::Html(html) => html.export(posts, config),
        }
    }
}

impl FromStr for ExporterKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<ExporterKind> {
        match s.to_lowercase().as_str() {
            "html" => Ok(ExporterKind::Html(html::HtmlExporter {})),
            _ => Err(Error::UnknownExporter(s.to_string())),
        }
    }
}
