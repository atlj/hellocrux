use log::warn;

use crate::crawl::{media_file::crawl_potential_media_file, subtitles::crawl_subtitles_dir};

use super::Result;
use std::{collections::HashMap, path::Path};

pub(super) async fn crawl_series_content(
    at: impl AsRef<Path>,
) -> Result<
    Option<(
        domain::SeriesContents,
        Vec<(domain::series::EpisodeIdentifier, domain::MediaPaths)>,
    )>,
> {
    let read_dir = crate::dir::fully_read_dir(at).await?;

    let crawl_futures = read_dir.map(|dir_item| {
        let path = dir_item.path();
        async move {
            let result = crawl_season(&path).await;
            (path, result)
        }
    });

    let crawl_results = futures::future::join_all(crawl_futures).await;

    let (series_contents, items_to_be_prepared) = crawl_results
        .into_iter()
        .inspect(|(path, result)| {
            if let Err(err) = result {
                warn!("Couldn't crawl season at {}: {err}", path.display())
            }
        })
        .flat_map(|(_, result)| result)
        .flatten()
        .fold(
            (HashMap::new(), Vec::with_capacity(20)),
            |(mut series_contents, mut all_episodes_to_prepare),
             (season_no, season_contents, episodes_to_prepare)| {
                if !season_contents.is_empty() {
                    series_contents.insert(season_no, season_contents);
                }
                all_episodes_to_prepare.extend(episodes_to_prepare);

                (series_contents, all_episodes_to_prepare)
            },
        );

    if series_contents.is_empty() && items_to_be_prepared.is_empty() {
        return Ok(None);
    }

    Ok(Some((series_contents, items_to_be_prepared)))
}

async fn crawl_season(
    at: impl AsRef<Path>,
) -> Result<
    Option<(
        u32,
        domain::SeasonContents,
        Vec<(domain::series::EpisodeIdentifier, domain::MediaPaths)>,
    )>,
> {
    // 1. Get the season number and make sure we're a dir
    let attributes = tokio::fs::metadata(&at).await?;
    if !attributes.is_dir() {
        return Ok(None);
    }

    let Some(season_path) = at
        .as_ref()
        .file_name()
        .and_then(|season_path| season_path.to_str())
    else {
        return Ok(None);
    };

    let Some(season_no) = super::get_numeric_content(season_path) else {
        return Ok(None);
    };

    // 2. Get subtitles repository
    let subtitles = crawl_subtitles_dir(&at).await?;

    // 3. Read all files then try parsing them each as episodes
    let crawl_futures = crate::dir::fully_read_dir(&at).await?.map(|dir_item| {
        let path = dir_item.path();
        let subtitles_ref = subtitles.as_deref();

        async move {
            let result = crawl_potential_media_file(&path, subtitles_ref, Some(season_no)).await;
            (path, result)
        }
    });

    let crawl_results = futures::future::join_all(crawl_futures).await;

    let (season_contents, episodes_to_prepare) = crawl_results
        .into_iter()
        .inspect(|(path, crawl_result)| {
            if let Err(err) = crawl_result {
                warn!("Couldn't crawl episode at {}: {err:#?}", path.display())
            }
        })
        .filter_map(|(_, result)| match result {
            Ok(Some((paths, Some(episode_identifier), needs_to_be_prepared))) => {
                Some((paths, episode_identifier, needs_to_be_prepared))
            }
            _ => None,
        })
        .fold(
            (HashMap::new(), Vec::with_capacity(20)),
            |(mut series_contents, mut episodes_to_prepare),
             (paths, episode_identifier, needs_to_be_prepared)| {
                if needs_to_be_prepared {
                    episodes_to_prepare.push((episode_identifier, paths));
                } else {
                    series_contents.insert(episode_identifier.episode_no, paths);
                }
                (series_contents, episodes_to_prepare)
            },
        );

    Ok(Some((season_no, season_contents, episodes_to_prepare)))
}
