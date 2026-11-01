use std::path::Path;

use crate::crawl::subtitles::grab_subtitles;

use super::Result;
use domain::subtitles::Subtitle;

pub(super) async fn crawl_potential_media_file(
    at: impl AsRef<Path>,
    subtitle_repo: Option<&[(Option<u32>, Subtitle)]>,
    season_no: Option<u32>,
) -> Result<
    Option<(
        domain::MediaPaths,
        Option<domain::series::EpisodeIdentifier>,
        bool,
    )>,
> {
    // 1. Make sure file exists and is a media file
    {
        if !domain::format::is_video_file(&at) {
            return Ok(None);
        }

        let attributes = tokio::fs::metadata(&at).await?;
        if !attributes.is_file() {
            return Ok(None);
        }
    }

    let Some(file_stem) = at.as_ref().file_stem().and_then(|stem| stem.to_str()) else {
        // No file stem = no media
        return Ok(None);
    };

    let media_path = at.as_ref().to_string_lossy().to_string();

    let Some((track_name, episode_identifier)) = (match season_no {
        Some(season_no) => {
            let (episode_number, encoded_part) =
                file_stem.split_once('-').unwrap_or((file_stem, file_stem));

            episode_number
                .parse::<u32>()
                .ok()
                .map(|episode_number| {
                    (
                        encoded_part,
                        domain::series::EpisodeIdentifier {
                            season_no,
                            episode_no: episode_number,
                        },
                    )
                })
                .map(|(encoded_part, episode_identifier)| {
                    let track_name = domain::encode_decode::decode_url_safe(encoded_part)
                        .unwrap_or_else(|_| encoded_part.to_string());
                    (track_name, Some(episode_identifier))
                })
        }
        None => {
            let track_name = domain::encode_decode::decode_url_safe(file_stem)
                .unwrap_or_else(|_| file_stem.to_string());
            Some((track_name, None))
        }
    }) else {
        return Ok(None);
    };

    // 2. Grab the subtitles
    let subtitles = match subtitle_repo {
        Some(subtitle_repo) => grab_subtitles(subtitle_repo, episode_identifier.as_ref()).await,
        None => Vec::with_capacity(0),
    };

    let media_paths = domain::MediaPaths {
        media: media_path,
        track_name,
        subtitles,
    };

    // 3. Check if file needs to be prepared
    let needs_to_be_prepared = crate::prepare::needs_to_be_prepared(&media_paths).await?;

    Ok(Some((
        media_paths,
        episode_identifier,
        needs_to_be_prepared,
    )))
}
