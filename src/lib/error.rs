use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Ureq(ureq::Error),
    ParseInt(std::num::ParseIntError),
    HttpClientError(u16),
    DownloadFailed { url: String, tries: u32 },
    MissingElement(&'static str),
    EmptyFile(String),
    NoPosts,
    UnknownExporter(String),
    ChannelSend,
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Ureq(e) => write!(f, "network error: {e}"),
            Error::ParseInt(e) => write!(f, "parse error: {e}"),
            Error::HttpClientError(code) => write!(f, "client error: {code}"),
            Error::DownloadFailed { url, tries } => {
                write!(f, "failed to download {url} after {tries} tries")
            }
            Error::MissingElement(name) => write!(f, "missing element: {name}"),
            Error::EmptyFile(url) => write!(f, "empty file: {url}"),
            Error::NoPosts => write!(f, "no posts to export"),
            Error::UnknownExporter(name) => write!(f, "unknown exporter: {name}"),
            Error::ChannelSend => write!(f, "failed to send event"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Ureq(e) => Some(e),
            Error::ParseInt(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e) }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self { Error::Ureq(e) }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self { Error::ParseInt(e) }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Self { Error::ChannelSend }
}
