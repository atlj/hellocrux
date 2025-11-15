use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::prepare::{Error, Result};
use domain::{
    MediaMetaData,
    series::{EditSeriesFileMappingForm, file_mapping_form_state},
};
use tokio::io::AsyncWriteExt;

pub async fn generate_series_media(
    media_dir: &Path,
    source_dir: &Path,
    mapping: EditSeriesFileMappingForm<file_mapping_form_state::Valid>,
    metadata: &MediaMetaData,
) -> Result<Box<[PathBuf]>> {
    let target_dir = media_dir.join(&metadata.title);

    // 1. Create destination dir
    // TODO consider extracting this
    {
        tokio::fs::create_dir_all(&target_dir)
            .await
            .map_err(|err| {
                Error::MoveError(
                    format!(
                        "Couldn't generate media dir at {}. Reason: {err}",
                        media_dir.display()
                    )
                    .into(),
                )
            })?;
    }

    let resolved_mapping: HashMap<PathBuf, PathBuf> = mapping
        .file_mapping
        .iter()
        .map(|(source_path, episode_identifier)| (source_dir.join(source_path), episode_identifier))
        .fold(
            HashMap::with_capacity(mapping.file_mapping.len()),
            |mut map, (source_path, episode_identifier)| {
                // TODO remove unwrap
                let file_name = source_path.file_name().unwrap();

                // TODO extract this logic
                let destination = target_dir.join(format!(
                    "{}/{}-{}",
                    episode_identifier.season_no,
                    episode_identifier.episode_no,
                    file_name.to_string_lossy()
                ));

                map.insert(source_path.clone(), destination);
                map
            },
        );

    let source_paths: Box<_> = mapping
        .file_mapping
        .into_keys()
        .map(|path| source_dir.join(path))
        .collect();

    // 2. Move media files to destination
    {
        // 2a. Make sure all file paths are valid
        let failed_paths: Box<_> = {
            let file_paths_exist_futures = source_paths.into_iter().map(async |file_path| {
                let result = tokio::fs::try_exists(&file_path).await;

                (file_path, result)
            });

            futures::future::join_all(file_paths_exist_futures)
                .await
                .into_iter()
                .filter_map(|(path, result)| match result {
                    Ok(true) => None,
                    _ => Some(path),
                })
                .collect()
        };

        if !failed_paths.is_empty() {
            return Err(Error::MoveError(
                format!(
                    "Some paths from series files mapping don't exist: {:#?}",
                    failed_paths
                )
                .into(),
            ));
        }

        // 2b. Move files
        let move_futures = resolved_mapping.iter().map(|(source, destination)| {
            async move {
                if let Some(parent) = destination.parent() {
                    // TODO log this
                    let _ = tokio::fs::create_dir_all(parent).await;
                }

                tokio::fs::rename(source, destination)
                    .await
                    .map_err(|err| {
                        Error::MoveError(
                            format!(
                                "Couldn't move movie file from {} to {}. Reason: {err}",
                                source.display(),
                                destination.display()
                            )
                            .into(),
                        )
                    })?;

                let result: Result<()> = Ok(());
                result
            }
        });

        let move_errors: Box<[Error]> = futures::future::join_all(move_futures)
            .await
            .into_iter()
            .filter_map(Result::err)
            .collect();

        if !move_errors.is_empty() {
            // Delete all changes
            // TODO maybe log this or return err
            let _ = tokio::fs::remove_dir_all(media_dir).await;

            let error = Error::MoveError(
                format!(
                    "Couldn't move files in series mapping due to error(s): {:#?}",
                    move_errors
                )
                .into(),
            );

            return Err(error);
        }
    }

    // 3. Save metadata
    // TODO extract this
    {
        let mut metadata_file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(target_dir.join("meta.json"))
            .await
            .map_err(|err| {
                Error::MoveError(format!("Can't open meta.json. Reason: {err}",).into())
            })?;

        let metadata_string = serde_json::to_string_pretty(metadata).map_err(|err| {
            Error::MoveError(format!("Can't serialize metadata into json. Reason: {err}").into())
        })?;

        metadata_file
            .write(metadata_string.as_bytes())
            .await
            .map_err(|err| {
                Error::MoveError(format!("Couldn't save metadata. Reason: {err}").into())
            })?;
    }

    Ok(resolved_mapping.into_values().collect())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs, io,
        marker::PhantomData,
        path::{Path, PathBuf},
    };

    use domain::{
        MediaMetaData,
        series::{EditSeriesFileMappingForm, EpisodeIdentifier},
    };

    use crate::prepare::moving::series::generate_series_media;

    #[tokio::test]
    async fn test_generate_series_media() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/prepare_seris")).await;
        tokio::fs::create_dir_all(test_data_path.join("tmp/prepare_series"))
            .await
            .unwrap();

        copy_dir_all(
            test_data_path.join("prepare_series"),
            test_data_path.join("tmp/prepare_series"),
        )
        .unwrap();

        let mapping = {
            let mut file_mapping = HashMap::new();
            file_mapping.insert(
                "season1/the-looks-S1E1.mov".to_string(),
                EpisodeIdentifier {
                    season_no: 1,
                    episode_no: 1,
                },
            );
            file_mapping.insert(
                "season1/the-looks-S1E2.mov".to_string(),
                EpisodeIdentifier {
                    season_no: 1,
                    episode_no: 2,
                },
            );
            file_mapping.insert(
                "season2/the-looks-S2E1.mov".to_string(),
                EpisodeIdentifier {
                    season_no: 2,
                    episode_no: 1,
                },
            );

            EditSeriesFileMappingForm {
                id: "hey".into(),
                file_mapping,
                phantom: PhantomData,
            }
        };

        let resulting_paths = generate_series_media(
            &test_data_path.join("tmp/series_media"),
            &test_data_path.join("tmp/prepare_series"),
            mapping,
            &MediaMetaData {
                title: "My Series".to_string(),
                thumbnail: "http://image.com".to_string(),
            },
        )
        .await
        .unwrap();

        dbg!(&resulting_paths);

        assert!(resulting_paths.into_iter().any(|path| {
            path.to_str().unwrap().contains(
                test_data_path
                    .join("tmp/series_media/My Series/1/1-the-looks-S1E1.mov")
                    .to_str()
                    .unwrap(),
            )
        }));

        assert!(
            tokio::fs::try_exists(
                test_data_path.join("tmp/series_media/My Series/1/1-the-looks-S1E1.mov")
            )
            .await
            .unwrap()
        );

        assert!(
            tokio::fs::try_exists(
                test_data_path.join("tmp/series_media/My Series/1/2-the-looks-S1E2.mov")
            )
            .await
            .unwrap()
        );

        assert!(
            tokio::fs::try_exists(
                test_data_path.join("tmp/series_media/My Series/2/1-the-looks-S2E1.mov")
            )
            .await
            .unwrap()
        );

        assert!(
            !tokio::fs::try_exists(
                test_data_path.join("tmp/series_media/My Series/1/random-file.txt")
            )
            .await
            .unwrap()
        );
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
