use domain::{SeriesContents, subtitles::Subtitle};
use log::error;

use crate::crawl::subtitles::remux_with_subtitles_if_missing;

use super::{Error, Result};
use std::{collections::HashMap, path::Path};

pub(super) async fn try_extract_series(
    path: impl AsRef<Path>,
) -> Result<Option<domain::SeriesContents>> {
    let read_dir = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let seasons_futures = read_dir.map(|entry| async move {
        let current_path = entry.path();
        if current_path.is_file() {
            let result: super::Result<Option<(u32, domain::SeasonContents)>> = Ok(None);
            return result;
        }
        let result = match super::get_numeric_content(entry.file_name().to_string_lossy().as_ref())
        {
            None => None,
            Some(season_no) => try_extract_season(entry.path())
                .await?
                .map(|season_contents| (season_no, season_contents)),
        };

        Ok(result)
    });

    let seasons = futures::future::join_all(seasons_futures).await;

    let result: SeriesContents = seasons
        .into_iter()
        .flat_map(|result| match result {
            Ok(val) => val.map(Ok),
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok(Some(result))
}

async fn try_extract_season(
    season_path: impl AsRef<Path>,
) -> Result<Option<domain::SeasonContents>> {
    let read_dir = crate::dir::fully_read_dir(&season_path)
        .await
        .map_err(|_| Error::CantReadDir(season_path.as_ref().into()))?;

    let all_subtitles = crate::crawl::subtitles::extract_subtitles(&season_path).await?;
    let len = all_subtitles.len();
    let mut subtitles_map: HashMap<u32, Vec<Subtitle>> = all_subtitles
        .into_iter()
        .flat_map(|subtitle| {
            let Some(episode_no) = get_episode_no(&subtitle.path) else {
                error!(
                    "Subtitle at {} has no episode no. Ignoring it.",
                    &subtitle.path
                );
                return None;
            };
            Some((episode_no, subtitle))
        })
        .fold(
            HashMap::with_capacity(len),
            |mut map, (episode_no, subtitle)| {
                let entry = map.entry(episode_no).or_insert(Vec::new());
                entry.push(subtitle);
                map
            },
        );

    let result: domain::SeasonContents = read_dir.fold(HashMap::new(), |mut map, entry| {
        let current_path = entry.path();
        if !domain::format::is_supported_video_file(&current_path) {
            return map;
        }

        if let Some(episode_no) =
            super::get_numeric_content(entry.file_name().to_string_lossy().as_ref())
        {
            let subtitles = subtitles_map
                .remove(&episode_no.try_into().unwrap())
                .unwrap_or_default();

            let media = current_path.to_string_lossy().into();

            let file_stem = current_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("Media file to have a valid stem");
            let track_name =
                get_episode_track_name(file_stem).unwrap_or_else(|| file_stem.to_string());

            let media_paths = domain::MediaPaths {
                subtitles,
                media,
                track_name,
            };

            map.insert(episode_no, media_paths);
        }

        map
    });

    let remux_futures = result
        .values()
        .map(|episode| remux_with_subtitles_if_missing(&episode.media, &episode.subtitles));

    futures::future::join_all(remux_futures).await;

    if result.is_empty() {
        return Ok(None);
    }

    Ok(Some(result))
}

fn get_episode_track_name(file_stem: &str) -> Option<String> {
    let (episode_number, encoded_part) = file_stem.split_once('-')?;

    if episode_number.parse::<usize>().is_err() {
        return None;
    }

    domain::encode_decode::decode_url_safe(encoded_part).ok()
}

fn get_episode_no(srt_path: impl AsRef<Path>) -> Option<u32> {
    let file_stem = srt_path
        .as_ref()
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .expect("Subtitle to have valid stem");

    let (episode_candidate, _) = file_stem.split_once('-')?;
    episode_candidate.parse().ok()
}

#[cfg(test)]
mod tests {
    use crate::crawl::series::{try_extract_season, try_extract_series};
    use crate::test_utils::fixtures_path;

    #[tokio::test]
    async fn extract_series() {
        let path = fixtures_path().join("crawl/example_series");
        let result = try_extract_series(&path).await.unwrap().unwrap();

        assert!(
            result
                .get(&1)
                .unwrap()
                .get(&1)
                .unwrap()
                .media
                .contains("1.mp4")
        );
        assert!(
            result
                .get(&1)
                .unwrap()
                .get(&2)
                .unwrap()
                .media
                .contains("2.mp4")
        );

        assert!(!result.contains_key(&2));
        assert!(
            result
                .get(&7)
                .unwrap()
                .get(&9)
                .unwrap()
                .media
                .contains("9.mp4")
        );

        assert!(!result.contains_key(&9));
    }

    #[tokio::test]
    async fn extract_season() {
        let path = fixtures_path().join("crawl/example_series");
        let result = try_extract_season(path.join("1")).await.unwrap().unwrap();
        let first_episode = result.get(&1).unwrap();
        assert!(first_episode.media.contains("1.mp4"));

        let subtitles = first_episode.subtitles.first().unwrap();
        assert!(subtitles.path.contains("turheyyyy.srt"));
        assert_eq!(subtitles.language, domain::language::LanguageCode::Turkish);
    }
}
