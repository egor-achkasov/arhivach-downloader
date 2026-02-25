#[derive(Debug, Clone)]
pub struct Config {
    pub urls: Vec<String>,
    pub thumb: bool,
    pub files: bool,
    pub resume: bool,
}
