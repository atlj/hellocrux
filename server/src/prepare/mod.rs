use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use domain::{
    MediaMetaData,
    series::{EditSeriesFileMappingForm, file_mapping_form_state},
};
use log::info;

mod convert;
mod moving;

pub async fn prepare_movie(
    media_dir: &Path,
    source_dir: &Path,
    metadata: &MediaMetaData,
) -> Result<()> {
    // TODO consider using mappings for movies.
    // 1. Find movie file
    let movie_file = find_movie_file(source_dir)
        .await?
        .ok_or(Error::PrepareError(
            format!("No movie file found at {}", source_dir.display()).into(),
        ))?;

    // 2. Move movie media and generate metadata
    let moved_file = moving::generate_movie_media(media_dir, &movie_file, metadata).await?;

    // 3. Convert if needed
    {
        if convert::should_convert(&moved_file) {
            info!(
                "Movie file with path {} is going to be converted.",
                moved_file.display()
            );

            convert::convert_media(
                &moved_file,
                &moved_file
                    .with_file_name(
                        moved_file
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace("-tbd", ""),
                    )
                    .with_extension("mp4"),
            )
            .await?;

            // 3a. Delete old file
            tokio::fs::remove_file(&moved_file).await.map_err(|err| {
            Error::PrepareError(
                format!(
                    "Converted a movie file but couldn't delete the source file at {}. Reason: {err}",
                    movie_file.display()
                )
                .into(),
            )
        })?;

            info!("Converted movie file at {}.", moved_file.display());
        }
    }

    Ok(())
}

// TODO reorder the params so they are unified
pub async fn prepare_series(
    media_dir: &Path,
    source_dir: &Path,
    metadata: &MediaMetaData,
    mapping: EditSeriesFileMappingForm<file_mapping_form_state::Valid>,
) -> Result<()> {
    // 1. Move movie media and generate metadata
    let resulting_paths =
        moving::generate_series_media(media_dir, source_dir, mapping, metadata).await?;

    // 2. Convert if needed
    {
        let conversion_futures = resulting_paths
            .into_iter()
            .map(|resulting_path| async move {
                if !convert::should_convert(&resulting_path) {
                    let result: Result<()> = Ok(());
                    return result
                };

                info!(
                    "Series file with path {} is going to be converted.",
                    resulting_path.display()
                );

                convert::convert_media(
                    &resulting_path,
                    // TODO REMOVE UNWRAP
                    &resulting_path.with_file_name(resulting_path.file_name().unwrap().to_str().unwrap().replace("-tbd", "")).with_extension("mp4")
                )
                    .await?;

                tokio::fs::remove_file(&resulting_path).await.map_err(|err| {
                    Error::PrepareError(
                        format!(
                            "Converted a movie file but couldn't delete the source file at {}. Reason: {err}",
                            resulting_path.display()
                        )
                        .into(),
                    )
                })?;

                info!(
                    "Converted series file at {}.",
                    resulting_path.display()
                );

                Ok(())
            });

        let conversion_errors: Box<_> = futures::future::join_all(conversion_futures)
            .await
            .into_iter()
            .flat_map(|result| result.err())
            .collect();

        if !conversion_errors.is_empty() {
            return Err(Error::PrepareError(
                format!(
                    "Couldn't convert all files from series. Reasons: {:#?}",
                    conversion_errors
                )
                .into(),
            ));
        }
    }

    Ok(())
}

async fn find_movie_file(source_dir: &Path) -> Result<Option<PathBuf>> {
    let mut read_dir = tokio::fs::read_dir(source_dir).await.map_err(|err| {
        Error::PrepareError(
            format!(
                "Can't read source dir at {}. Reason: {err}",
                source_dir.display()
            )
            .into(),
        )
    })?;

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();

        if path.is_dir()
            && let Some(sub_dir_media_path) = Box::pin(find_movie_file(&path)).await?
        {
            return Ok(Some(sub_dir_media_path));
        }

        if domain::format::is_video_file(&path) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::OpenOptions,
        io::Read,
    };

    use domain::MediaMetaData;

    use crate::prepare::{find_movie_file, prepare_movie, prepare_series};
    use crate::test_utils::{Fixture, episode_mapping, fixture_sandbox, fixtures_path};

    #[tokio::test]
    async fn test_prepare_movie() {
        let (sandbox, source) = fixture_sandbox(Fixture::Prepare);
        let output_dir = sandbox.path().join("output");

        let metadata = MediaMetaData {
            title: "Jellyfish".to_string(),
            thumbnail: "https://some-link".to_string(),
        };

        prepare_movie(&output_dir, &source, &metadata)
            .await
            .unwrap();

        tokio::fs::try_exists(output_dir.join("Jellyfish/movie.mov"))
            .await
            .unwrap();

        let meta_file_contents = {
            let mut meta_file = OpenOptions::new()
                .read(true)
                .open(output_dir.join("Jellyfish/meta.json"))
                .unwrap();
            let mut string = String::new();
            meta_file.read_to_string(&mut string).unwrap();
            string
        };

        let saved_metadata: MediaMetaData = serde_json::from_str(&meta_file_contents).unwrap();

        assert_eq!(metadata, saved_metadata);
    }

    #[tokio::test]
    async fn test_prepare_series() {
        let (sandbox, source) = fixture_sandbox(Fixture::PrepareSeries);
        let output_dir = sandbox.path().join("output");

        let metadata = MediaMetaData {
            title: "Amazing Series".to_string(),
            thumbnail: "https://some-link".to_string(),
        };

        let mapping = episode_mapping("hey", &[
            ("season1/the-looks-S1E1.mkv", 1, 1),
            ("season1/the-looks-S1E2.mkv", 1, 2),
            ("season2/the-looks-S2E1.mkv", 2, 1),
        ]);

        prepare_series(&output_dir, &source, &metadata, mapping)
            .await
            .unwrap();

        let meta_file_contents = {
            let mut meta_file = OpenOptions::new()
                .read(true)
                .open(output_dir.join("Amazing_Series/meta.json"))
                .unwrap();
            let mut string = String::new();
            meta_file.read_to_string(&mut string).unwrap();
            string
        };

        let saved_metadata: MediaMetaData = serde_json::from_str(&meta_file_contents).unwrap();

        assert_eq!(metadata, saved_metadata);

        assert!(
            tokio::fs::try_exists(
                output_dir.join("Amazing_Series/1/1-dGhlLWxvb2tzLVMxRTE=.mp4")
            )
            .await
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_find_movie_file() {
        let fixtures = fixtures_path();

        assert_eq!(
            find_movie_file(&fixtures.join("check_video_file/mkv"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "file.mkv"
        );

        assert_eq!(
            find_movie_file(&fixtures.join("check_video_file/mov"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "hey.mov"
        );

        assert_eq!(
            find_movie_file(&fixtures.join("check_video_file/mp4"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "file.mp4"
        );

        assert_eq!(
            find_movie_file(&fixtures.join("check_video_file/nested"))
                .await
                .unwrap()
                .unwrap()
                .file_name()
                .unwrap(),
            "imavideo.mp4"
        );

        assert!(
            find_movie_file(&fixtures.join("check_video_file/none"))
                .await
                .unwrap()
                .is_none()
        );
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MoveError(Box<str>),
    ConvertError(Box<str>),
    PrepareError(Box<str>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl From<crate::ffmpeg::Error> for Error {
    fn from(value: crate::ffmpeg::Error) -> Self {
        Self::ConvertError(value.to_string().into_boxed_str())
    }
}
