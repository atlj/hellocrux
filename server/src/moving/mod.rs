mod movies;
mod series;

use std::path::PathBuf;

pub use movies::generate_movie_media;
pub use series::generate_series_media;

const URL_SAFE_NON_ALPHANUMERIC_CHARS: [char; 11] =
    ['$', '-', '_', '.', '+', '!', '*', '\'', '(', ')', ','];

fn sanitize_name_for_url(input: &str) -> String {
    input
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() | URL_SAFE_NON_ALPHANUMERIC_CHARS.contains(&char) {
                char
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Media file has no name")]
    NoFileName,
    #[error("Media file has no extension")]
    NoExtension,
    #[error("Coudln't create dir for media at {path}. {inner}")]
    CantCreateDir {
        path: PathBuf,
        inner: std::io::Error,
    },
    #[error("Couldn't move media from {from} to {to}. {inner}")]
    CantMove {
        from: PathBuf,
        to: PathBuf,
        inner: std::io::Error,
    },
    #[error("Can't create metadata at {at}. {inner}")]
    CantCreateMetaData { at: PathBuf, inner: std::io::Error },
    #[error("Can't serialize metadata {metadata:#?}. {inner}")]
    CantSerializeMetadata {
        metadata: domain::MediaMetaData,
        inner: serde_json::Error,
    },
    #[error("Can't write metadata at {at}. {inner}")]
    CantWriteMetadata { at: PathBuf, inner: std::io::Error },
}

type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use crate::moving::sanitize_name_for_url;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_name_for_url("Hello World"), "Hello_World");
        assert_eq!(sanitize_name_for_url("valid"), "valid");
        assert_eq!(sanitize_name_for_url("|nvalid"), "_nvalid");
        assert_eq!(sanitize_name_for_url("bo$$"), "bo$$");
        assert_eq!(sanitize_name_for_url("Co!!"), "Co!!");
        assert_eq!(sanitize_name_for_url(":nvalid"), "_nvalid");
    }
}
