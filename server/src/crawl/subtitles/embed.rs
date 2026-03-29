use std::path::Path;

use domain::subtitles::Subtitle;
use log::info;

async fn embed_subtitles_if_missing(
    movie_path: impl AsRef<Path>,
    subtitles: &[Subtitle],
) -> ffmpeg::Result<()> {
    // 1. Get all ids in subtitles
    let subtitle_ids = subtitles
        .iter()
        .flat_map(|subtitle| get_subtitle_id(&subtitle.path))
        .collect::<Vec<_>>();

    // 2. Get the tracks in movie path
    let tracks = ffmpeg::get_tracks(&movie_path).await?;

    // 3. Make sure all ids match
    let missing_a_subtitle = tracks
        .flat_map(|track| track.map(|track| track.id().to_string()))
        .any(|id| !subtitle_ids.contains(&id.as_str()));

    if !missing_a_subtitle {
        return Ok(());
    }

    info!(
        "Movie at {} is missing some subtitles. Remuxing it.",
        movie_path.as_ref().display()
    );

    embed_subtitles(movie_path, subtitles).await
}

/// Embed subtitles to a movie track
///
/// This will pick the video and audio tracks from the movie file then embed the subtitles
/// on top of them. Any existing subtitle track in movie file will be ignored.
async fn embed_subtitles(
    movie_path: impl AsRef<Path>,
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

    ffmpeg::encode_video(tracks, movie_path.as_ref().to_path_buf()).await?;

    Ok(())
}

fn get_subtitle_id(path: &impl AsRef<Path>) -> Option<&str> {
    let file_name = path.as_ref().file_name()?.to_str()?;
    let (_, id) = file_name.rsplit_once('-')?;
    Some(id)
}
