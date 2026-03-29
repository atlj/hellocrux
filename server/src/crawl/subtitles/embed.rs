use std::path::Path;

use domain::subtitles::Subtitle;

/// Embed subtitles to a movie track
///
/// This will pick the video and audio tracks from the movie file then embed the subtitles
/// on top of them. Any existing subtitle track in movie file will be ignored.
async fn embed_subtitles(
    movie_path: impl AsRef<Path>,
    subtitles: impl Iterator<Item = Subtitle>,
) -> ffmpeg::Result<()> {
    let tracks_info = ffmpeg::get_tracks(&movie_path)
        .await?
        .collect::<Result<Vec<_>, _>>()?;

    let subtitle_tracks =
        subtitles.map(
            |Subtitle { language, path }| ffmpeg::TrackSelection::Subtitle {
                input_path: path.into(),
                track_id: 0,
                language: Some(language),
                external_id: None,
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
