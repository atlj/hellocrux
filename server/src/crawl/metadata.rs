use std::path::Path;

use super::{Error, Result};
use domain::MediaMetaData;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

pub(super) async fn read_metadata(path: impl AsRef<Path>) -> Result<MediaMetaData> {
    let metadata_string = {
        let mut metadata_file = OpenOptions::new()
            .read(true)
            .open(path.as_ref().join("meta.json"))
            .await
            .map_err(|_| Error::NoMetadata)?;

        let mut string = String::new();
        metadata_file
            .read_to_string(&mut string)
            .await
            .map_err(|_| Error::CorruptedMetadata)?;
        string
    };

    Ok(serde_json::from_str(&metadata_string)?)
}

#[cfg(test)]
mod tests {
    use crate::{crawl::metadata::read_metadata, test_utils::fixtures_path};

    #[tokio::test]
    async fn test_read_metadata() {
        let path = fixtures_path().join("crawl/example_movie");
        read_metadata(path).await.unwrap();
    }
}
