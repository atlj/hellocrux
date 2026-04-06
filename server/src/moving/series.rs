use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{Error, Result};
use domain::{
    MediaMetaData,
    series::{EditSeriesFileMappingForm, file_mapping_form_state},
};

use super::sanitize_name_for_url;

pub async fn generate_series_media(
    media_dir: &Path,
    source_dir: &Path,
    mapping: EditSeriesFileMappingForm<file_mapping_form_state::Valid>,
    metadata: &MediaMetaData,
) -> Result<Box<[PathBuf]>> {
    let target_dir = media_dir.join(sanitize_name_for_url(&metadata.title));

    // 1. Create destination dir
    {
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(|err| Error::CantCreateDir {
                path: target_dir.to_path_buf(),
                inner: err,
            })?;
    }

    let resolved_mapping: HashMap<PathBuf, PathBuf> = mapping
        .file_mapping
        .iter()
        .map(|(source_path, episode_identifier)| (source_dir.join(source_path), episode_identifier))
        .try_fold(
            HashMap::with_capacity(mapping.file_mapping.len()),
            |mut map, (source_path, episode_identifier)| {
                let encoded_file_name = {
                    let extension = source_path.extension().ok_or(Error::NoExtension)?;
                    let file_stem = source_path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .ok_or(Error::NoFileName)?;
                    let encoded_file_stem = domain::encode_decode::encode_url_safe(file_stem);

                    format!(
                        "{}/{}-{}.{}",
                        episode_identifier.season_no,
                        episode_identifier.episode_no,
                        encoded_file_stem,
                        extension.to_string_lossy()
                    )
                };

                let destination = target_dir.join(encoded_file_name);
                map.insert(source_path.clone(), destination);

                Ok(map)
            },
        )?;

    // 2. Move media files to destination
    {
        // 2a. Move files
        let move_futures = resolved_mapping
            .iter()
            .map(|(source, destination)| async move {
                if let Some(parent) = destination.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|err| {
                        Error::CantCreateDir {
                            path: destination
                                .parent()
                                .map(|parent| parent.to_path_buf())
                                .unwrap_or_default(),
                            inner: err,
                        }
                    })?;
                }

                tokio::fs::rename(source, destination)
                    .await
                    .map_err(|err| Error::CantMove {
                        from: source.clone(),
                        to: destination.clone(),
                        inner: err,
                    })?;

                let result: Result<()> = Ok(());
                result
            });

        futures::future::join_all(move_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
    }

    // 3. Save metadata
    super::metadata::save_metadata(&target_dir, metadata.clone()).await?;

    Ok(resolved_mapping.into_values().collect())
}

#[cfg(test)]
mod tests {
    use domain::MediaMetaData;

    use crate::{
        moving::generate_series_media,
        test_utils::{Fixture, episode_mapping, fixture_sandbox},
    };

    #[tokio::test]
    async fn test_generate_series_media() {
        let (sandbox, source) = fixture_sandbox(Fixture::PrepareSeries);

        let mapping = episode_mapping(
            "hey",
            &[
                ("season1/the-looks-S1E1.mkv", 1, 1),
                ("season1/the-looks-S1E2.mkv", 1, 2),
                ("season2/the-looks-S2E1.mkv", 2, 1),
            ],
        );

        let output_dir = sandbox.path().join("series_media");
        let resulting_paths = generate_series_media(
            &output_dir,
            &source,
            mapping,
            &MediaMetaData {
                title: "My Series".to_string(),
                thumbnail: "http://image.com".to_string(),
            },
        )
        .await
        .unwrap();

        let ep1 = output_dir.join("My_Series/1/1-dGhlLWxvb2tzLVMxRTE=-tbd.mkv");
        let ep2 = output_dir.join("My_Series/1/2-dGhlLWxvb2tzLVMxRTI=-tbd.mkv");
        let ep3 = output_dir.join("My_Series/2/1-dGhlLWxvb2tzLVMyRTE=-tbd.mkv");

        assert!(resulting_paths.iter().any(|p| p == &ep1));

        assert!(tokio::fs::try_exists(&ep1).await.unwrap());
        assert!(tokio::fs::try_exists(&ep2).await.unwrap());
        assert!(tokio::fs::try_exists(&ep3).await.unwrap());

        assert!(
            !tokio::fs::try_exists(output_dir.join("My_Series/1/random-file.txt"))
                .await
                .unwrap()
        );
    }
}
