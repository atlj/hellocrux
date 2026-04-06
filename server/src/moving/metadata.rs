use tokio::io::AsyncWriteExt;

use super::{Error, Result};
use std::path::Path;

pub async fn save_metadata(
    at_folder: impl AsRef<Path>,
    metadata: domain::MediaMetaData,
) -> Result<()> {
    let destination = at_folder.as_ref().join("meta.json");
    let mut metadata_file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&destination)
        .await
        .map_err(|err| Error::CantCreateMetaData {
            at: destination.clone(),
            inner: err,
        })?;

    let metadata_string =
        serde_json::to_string_pretty(&metadata).map_err(|err| Error::CantSerializeMetadata {
            metadata,
            inner: err,
        })?;

    metadata_file
        .write(metadata_string.as_bytes())
        .await
        .map_err(|err| Error::CantWriteMetadata {
            at: destination,
            inner: err,
        })?;

    Ok(())
}
