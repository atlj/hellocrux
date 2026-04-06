use std::path::{Path, PathBuf};

use log::info;

pub async fn prepare_media(
    media_identifier: &domain::MediaIdentifier,
    track_selections: impl IntoIterator<Item = ffmpeg::TrackSelection>,
) -> Result<()> {
    let media_path: &Path = media_identifier.path().media.as_ref();
    let should_override_container = media_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(domain::is_container_compatible)
        .unwrap_or(true);

    // TODO ensure all track selections are included in tracks

    // Not each track needs to be converted, then just use `copy` to remux
    let converted_tracks = track_selections
        .into_iter()
        .map(|selection| match &selection {
            ffmpeg::TrackSelection::Video { codec, .. } => {
                if domain::is_video_codec_compatible(codec) {
                    selection.with_codec(domain::DEFAULT_VIDEO_CODEC.to_string())
                } else {
                    selection.with_codec("copy".to_string())
                }
            }
            ffmpeg::TrackSelection::Audio { codec, .. } => {
                if domain::is_audio_codec_compatible(codec) {
                    selection.with_codec(domain::DEFAULT_AUDIO_CODEC.to_string())
                } else {
                    selection.with_codec("copy".to_string())
                }
            }
            _ => selection,
        });

    // TODO also embed subtitles here

    let temp_dir = tempfile::tempdir()?;
    let file_stem: &Path = media_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("media")
        .as_ref();
    let extension = if should_override_container {
        "mp4"
    } else {
        media_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(domain::DEFAULT_CONTAINER_FORMAT)
    };

    let temp_path = temp_dir
        .path()
        .join(file_stem.with_added_extension(extension));

    ffmpeg::encode_video(converted_tracks.collect(), &temp_path).await?;

    tokio::fs::copy(&temp_path, media_path)
        .await
        .map_err(|inner| Error::CantCopy {
            from: temp_path,
            to: media_path.to_path_buf(),
            inner,
        })?;

    // If we override container, original file still exists since
    // copy didn't override it
    if should_override_container {
        tokio::fs::remove_file(media_path)
            .await
            .map_err(Error::CantDeleteOriginal)?
    }

    Ok(())
}

/// Checks whether media needs to be prepared for compatibility.
///
/// If this returns `true`, pass the media to `prepare` service.
pub async fn needs_to_be_prepared(path: impl AsRef<Path>) -> Result<bool> {
    // 1. Check container
    let is_container_compatible = path
        .as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(domain::is_container_compatible)
        .unwrap_or(false);

    if !is_container_compatible {
        info!(
            "Media at {} needs to be prepared because its container isn't compatible",
            path.as_ref().display()
        );
        return Ok(true);
    }

    // 2. Check tracks
    let tracks = ffmpeg::get_tracks(&path).await?;
    for track in tracks {
        let track = track?;
        if !track.is_codec_compatible() {
            info!(
                "Media at {} needs to be prepared because one of its tracks ({track:#?}) isn't compatible.",
                path.as_ref().display()
            );
            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn default_track_selections(
    media_path: impl AsRef<Path>,
) -> Result<Option<Vec<ffmpeg::TrackSelection>>> {
    let tracks = ffmpeg::get_tracks(&media_path)
        .await?
        .collect::<core::result::Result<Vec<_>, _>>()?;

    let (video_track_count, audio_track_count) = tracks.iter().fold(
        (0_usize, 0_usize),
        |(video_track_count, audio_track_count), track| match track {
            domain::Track::Video { .. } => (video_track_count + 1, audio_track_count),
            domain::Track::Audio { .. } => (video_track_count, audio_track_count + 1),
            domain::Track::Subtitle { .. } => (video_track_count, audio_track_count),
        },
    );

    if video_track_count > 1 || audio_track_count > 1 {
        return Ok(None);
    }

    let selections = tracks
        .into_iter()
        .map(|track| ffmpeg::TrackExt::into_selection(track, media_path.as_ref().into()))
        .collect::<Vec<_>>();

    Ok(Some(selections))
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("There was a problem with ffmpeg {0}")]
    Ffmpeg(ffmpeg::Error),
    #[error("Couldn't create a temporary folder to put ffmpeg output {0}")]
    CantCreateTempDir(std::io::Error),
    #[error("Couldn't copy temp file from {from:#?} to {to:#?}. {inner}")]
    CantCopy {
        from: PathBuf,
        to: PathBuf,
        inner: std::io::Error,
    },
    #[error("Couldn't delete original, incompatible file. {0}")]
    CantDeleteOriginal(std::io::Error),
}

impl From<ffmpeg::Error> for Error {
    fn from(value: ffmpeg::Error) -> Self {
        Error::Ffmpeg(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::CantCreateTempDir(value)
    }
}
