use clap::Parser;
use anyhow::Result;

use std::path::PathBuf;

pub struct Config{
    pub urls: Vec<String>,
    pub thumb: bool,
    pub files: bool
}

pub fn parse_args() -> Result<Config> {
    #[derive(Parser)]
    #[command(about, long_about)]
    struct Cli {
        /// URL to download
        url: Option<String>,

        /// Path to a text file containing a list of URLs (one per line)
        #[arg(short = 'l', long = "list")]
        list: Option<PathBuf>,

        /// Download thumbnail images, default: false
        #[arg(short = 't', long = "thumb", default_value_t = false)]
        thumb: bool,

        /// Download files (images, videos, gifs, etc), default: false
        #[arg(short = 'f', long = "files", default_value_t = false)]
        files: bool,
    }
    let cli = Cli::parse();

    let mut urls = Vec::new();
    // [URL]    
    if let Some(url) = cli.url {
        urls.push(url);
    }
    // [List]
    if let Some(list) = cli.list {
        for line in std::fs::read_to_string(list)?.lines() {
            urls.push(line.to_string());
        }
    }
    if urls.is_empty() {
        anyhow::bail!("No URLs provided");
    }

    Ok(Config {
        urls,
        thumb: cli.thumb,
        files: cli.files
    })
}
