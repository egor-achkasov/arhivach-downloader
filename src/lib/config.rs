use crate::export::ExporterKind;

#[derive(Clone)]
pub struct Config {
    pub url: String,
    pub dir: std::path::PathBuf,
    pub exporter: ExporterKind,
    pub thumb: bool,
    pub files: bool,
    pub resume: bool,
    pub download_retries: u32,
}
