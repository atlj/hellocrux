use std::path::Path;

use domain::subtitles::Subtitle;
use log::{error, info};

/// Ensure subtitles are embedded, remux if not
///
/// Checks the available subtitles at the media_path, if there is a mismatch, remuxes the media file
/// with given subtitles.
/// There can be a mismatch if:
/// 1. There are subtitles present in slice but not in media
/// 2. There are subtitles present in media but not in subtitles
/// Both of this cases will lead to remuxing.
pub async fn remux_with_subtitles_if_missing(
    media_path: impl AsRef<Path> + std::fmt::Debug,
    subtitles: &[Subtitle],
) {
    let Some(file_name) = media_path
        .as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
    else {
        error!("Media path has no file stem: {media_path:#?}",);
        return;
    };

    let Some(extension) = media_path
        .as_ref()
        .extension()
        .and_then(|stem| stem.to_str())
    else {
        error!("Media path has no extension: {media_path:#?}",);
        return;
    };

    let Ok(temp_dir) = tempfile::tempdir() else {
        error!("Couldn't create a temp dir to embed subs for {media_path:#?}. Skipping this step.",);
        return;
    };

    let temp_file_name = format!("{file_name}-tmp.{extension}");
    let temp_file = temp_dir.path().join(temp_file_name);

    match embed_subtitles_if_missing(&media_path, &temp_file, &subtitles).await {
        Ok(did_embed) => {
            if !did_embed {
                return;
            }

            if let Err(error) = tokio::fs::copy(&temp_file, &media_path).await {
                error!(
                    "Couldn't move subtitle embedded file from {temp_file:#?} to {media_path:#?}. Reason: {error}"
                );
            }
        }
        Err(error) => {
            error!("Couldn't embed subtitles for {media_path:#?}. Reason: {error}")
        }
    }
}

/// Embed subtitles only if they are different
async fn embed_subtitles_if_missing(
    movie_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    subtitles: &[Subtitle],
) -> ffmpeg::Result<bool> {
    // 1. Get all ids in subtitles
    let subtitle_ids = subtitles
        .iter()
        .flat_map(|subtitle| get_subtitle_id(&subtitle.path))
        .collect::<Vec<_>>();

    // 2. Get the tracks in movie path
    let tracks = ffmpeg::get_tracks(&movie_path).await?.collect::<Vec<_>>();

    // 3. Make sure all ids match
    let missing_id = tracks
        .iter()
        .flat_map(|track| match track.as_ref().ok() {
            Some(ffmpeg::Track::Subtitle { external_id, .. }) => external_id.as_ref(),
            _ => None,
        })
        .find(|id| !subtitle_ids.contains(&id.as_str()));

    let sub_track_count = tracks
        .iter()
        .filter(|track| matches!(track, Ok(ffmpeg::Track::Subtitle { .. })))
        .count();

    let sub_file_count = subtitles.len();

    if missing_id.is_none() && sub_track_count == sub_file_count {
        return Ok(false);
    }

    info!(
        "Movie at {} is missing a subtitles. Missing id: {missing_id:#?}. Sub file count: {sub_file_count}. Sub track count: {sub_track_count}.Remuxing it.",
        movie_path.as_ref().display(),
    );

    embed_subtitles(movie_path, output_path, subtitles).await?;

    Ok(true)
}

/// Embed subtitles to a movie track
///
/// This will pick the video and audio tracks from the movie file then embed the subtitles
/// on top of them. Any existing subtitle track in movie file will be ignored.
async fn embed_subtitles(
    movie_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    subtitles: &[Subtitle],
) -> ffmpeg::Result<()> {
    let tracks_info = ffmpeg::get_tracks(&movie_path)
        .await?
        .collect::<Result<Vec<_>, _>>()?;

    let subtitle_tracks =
        subtitles.iter().map(
            |Subtitle { language, path }| ffmpeg::TrackSelection::Subtitle {
                input_path: path.into(),
                track_id: 0,
                language: Some(language.clone()),
                external_id: get_subtitle_id(&path).map(|str| str.to_string()),
            },
        );

    let tracks = tracks_info
        .into_iter()
        .filter_map(|track| {
            let input_path = movie_path.as_ref().to_path_buf();
            let track_id = *track.id();
            match track {
                ffmpeg::Track::Video { .. } => Some(ffmpeg::TrackSelection::Video {
                    input_path,
                    track_id,
                    codec: "copy".to_string(),
                }),
                ffmpeg::Track::Audio { .. } => Some(ffmpeg::TrackSelection::Audio {
                    input_path,
                    track_id,
                    codec: "copy".to_string(),
                }),
                _ => None,
            }
        })
        .chain(subtitle_tracks)
        .collect();

    ffmpeg::encode_video(tracks, output_path.as_ref().to_path_buf()).await?;

    Ok(())
}

fn get_subtitle_id(path: &impl AsRef<Path>) -> Option<&str> {
    let file_name = path.as_ref().file_stem()?.to_str()?;
    let (_, id) = file_name.rsplit_once('-')?;
    Some(id)
}
