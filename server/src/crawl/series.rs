use domain::{series::EpisodeIdentifier, subtitles::Subtitle};
use either::Either;
use log::error;

use super::{Error, Result};
use std::{collections::HashMap, path::Path};

pub(super) async fn crawl_series(
    id: String,
    path: impl AsRef<Path>,
) -> Result<Option<(Option<domain::SeriesContents>, Vec<domain::MediaIdentifier>)>> {
    let read_dir = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let crawl_season_futures = read_dir.flat_map(|entry| {
        let current_path = entry.path();
        if current_path.is_file() {
            return None;
        }

        let season_no = super::get_numeric_content(entry.file_name().to_string_lossy().as_ref())?;

        let id = id.clone();
        Some(async move { (season_no, crawl_season(id, season_no, entry.path()).await) })
    });

    let seasons = futures::future::join_all(crawl_season_futures).await;
    let len = seasons.len();

    let (seasons, to_prepare) = seasons.into_iter().try_fold(
        (HashMap::with_capacity(len), Vec::with_capacity(len)),
        |(mut season_map, mut prepare_vec), (season_no, result)| {
            let (season, to_prepare) = result?;

            if !season.is_empty() {
                season_map.insert(season_no, season);
            }

            prepare_vec.extend(to_prepare);

            Ok((season_map, prepare_vec)) as Result<_>
        },
    )?;

    Ok(Some((
        if seasons.is_empty() {
            None
        } else {
            Some(seasons)
        },
        to_prepare,
    )))
}

async fn crawl_season(
    id: String,
    season_no: u32,
    season_path: impl AsRef<Path>,
) -> Result<(domain::SeasonContents, Vec<domain::MediaIdentifier>)> {
    let read_dir = crate::dir::fully_read_dir(&season_path)
        .await
        .map_err(|_| Error::CantReadDir(season_path.as_ref().into()))?;

    let all_subtitles = crate::crawl::subtitles::crawl_subtitles(&season_path).await?;
    let len = all_subtitles.len();
    let mut subtitles_map: HashMap<u32, Vec<Subtitle>> = all_subtitles
        .into_iter()
        .flat_map(|(episode_no, subtitle)| {
            let Some(episode_no) = episode_no else {
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

    let episode_futures = read_dir.flat_map(|entry| {
        let episode_no = super::get_numeric_content(&entry.file_name().to_string_lossy())?;
        let episode = EpisodeIdentifier {
            season_no,
            episode_no,
        };

        let subtitles = subtitles_map.remove(&episode_no).unwrap_or_default();

        Some(extract_episode(
            id.clone(),
            episode,
            entry.path(),
            subtitles,
        ))
    });

    let episodes = futures::future::join_all(episode_futures).await.into_iter();
    let len = episodes.len();

    let result = episodes.into_iter().try_fold(
        (HashMap::with_capacity(len), Vec::with_capacity(len)),
        |(mut episodes, mut prepare), result| {
            let Some(item) = result? else {
                return Ok((episodes, prepare)) as Result<_>;
            };

            match item {
                Either::Left((episode_id, ready)) => {
                    episodes.insert(episode_id, ready);
                }
                Either::Right(need_to_prepare) => {
                    prepare.push(need_to_prepare);
                }
            }

            Ok((episodes, prepare))
        },
    )?;

    Ok(result)
}

async fn extract_episode(
    id: String,
    episode_identifier: EpisodeIdentifier,
    path: impl AsRef<Path>,
    subtitles: Vec<Subtitle>,
) -> Result<Option<Either<(u32, domain::MediaPaths), domain::MediaIdentifier>>> {
    if !domain::format::is_video_file(&path) {
        return Ok(None);
    }

    let path_string = path.as_ref().to_string_lossy().into();

    let file_stem = path
        .as_ref()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("Media file to have a valid stem");

    let track_name = get_episode_track_name(file_stem).unwrap_or_else(|| file_stem.to_string());

    let media_paths = domain::MediaPaths {
        subtitles,
        media: path_string,
        track_name,
    };

    if crate::prepare::needs_to_be_prepared(&media_paths)
        .await
        .map_err(Error::CantCheckCompatibility)?
    {
        return Ok(Some(Either::Right(domain::MediaIdentifier::Series {
            id,
            episode: episode_identifier,
            path: media_paths,
        })));
    };

    Ok(Some(Either::Left((
        episode_identifier.episode_no,
        media_paths,
    ))))
}

fn get_episode_track_name(file_stem: &str) -> Option<String> {
    let (episode_number, encoded_part) = file_stem.split_once('-')?;

    if episode_number.parse::<usize>().is_err() {
        return None;
    }

    domain::encode_decode::decode_url_safe(encoded_part).ok()
}
