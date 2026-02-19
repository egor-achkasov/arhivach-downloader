#[derive(Debug, Clone)]
pub struct File {
    /// original name, "videolol.mp4"
    pub name_orig: String,
    /// timestampname, "17699100670710.mp4"
    pub name_timestamp: String,
    /// thumbnail url, "https://archivach.vc/storage/t/aeaa7825f8d8ffe3f07f242a59b7761c.thumb"
    pub url_thumb: String,
    /// url, "https://i.arhivach.vc/storage/a/ea/aeaa7825f8d8ffe3f07f242a59b7761c.mp4"
    pub url: String,
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}]\n  url:   {}\n  thumb: {}",
            self.name_orig, self.name_timestamp, self.url, self.url_thumb
        )
    }
}