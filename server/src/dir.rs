use std::path::Path;

use tokio::fs::DirEntry;

pub(crate) async fn fully_read_dir(
    path: impl AsRef<Path>,
) -> tokio::io::Result<impl Iterator<Item = DirEntry>> {
    let mut result: Vec<DirEntry> = Vec::new();
    let mut read_dir = tokio::fs::read_dir(path).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        result.push(entry);
    }

    Ok(result.into_iter())
}
