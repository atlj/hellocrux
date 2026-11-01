use super::Result;
use std::path::Path;

use domain::{series::EpisodeIdentifier, subtitles::Subtitle};
use log::warn;

pub async fn grab_subtitles(
    subtitle_repo: &[(Option<u32>, Subtitle)],
    episode: Option<&EpisodeIdentifier>,
) -> Vec<Subtitle> {
    subtitle_repo
        .iter()
        .filter_map(
            |(subtitle_episode, subtitle)| match (&episode, subtitle_episode) {
                // Series episode and numbers are matching
                (Some(EpisodeIdentifier { episode_no, .. }), Some(subtitle_episode))
                    if episode_no == subtitle_episode =>
                {
                    Some(subtitle)
                }
                // Movie
                (None, _) => Some(subtitle),
                _ => None,
            },
        )
        .cloned()
        .collect()
}

/// Looks at `path/subtitles`
pub async fn crawl_subtitles_dir(
    at_parent_dir: impl AsRef<Path>,
) -> Result<Option<Vec<(Option<u32>, Subtitle)>>> {
    let dir = at_parent_dir.as_ref().join("subtitles");

    if !tokio::fs::try_exists(&dir).await? {
        return Ok(None);
    }

    let attributes = tokio::fs::metadata(&dir).await?;
    if !attributes.is_dir() {
        return Ok(None);
    }

    let read_dir = crate::dir::fully_read_dir(&dir).await?;

    let crawl_futures = read_dir.into_iter().map(|dir_item| {
        let path = dir_item.path();
        async move { (parse_subtitle_file(&path).await, path) }
    });

    let crawl_results = futures::future::join_all(crawl_futures).await;
    let results: Vec<_> = crawl_results
        .into_iter()
        .inspect(|(result, path)| {
            if let Err(err) = result {
                warn!("Couldn't crawl subtitle at {}: {err}", path.display())
            }
        })
        .flat_map(|(result, _)| result)
        .flatten()
        .collect();

    if results.is_empty() {
        return Ok(None);
    }

    Ok(Some(results))
}

async fn parse_subtitle_file(
    at: &impl AsRef<Path>,
) -> Result<
    Option<(
        // Episode no
        Option<u32>,
        Subtitle,
    )>,
> {
    let attributes = tokio::fs::metadata(at.as_ref()).await?;
    if !attributes.is_file() {
        return Ok(None);
    }

    let Some(file_stem) = at.as_ref().file_stem().map(|stem| stem.to_string_lossy()) else {
        return Ok(None);
    };

    let mut dash_iter = file_stem.split('-');
    let (episode_no_unparsed, language_code_unparsed, id_unparsed) =
        match (dash_iter.next(), dash_iter.next(), dash_iter.next()) {
            // episode_num, language, id
            (Some(episode_no), Some(language_code), Some(id)) => {
                (Some(episode_no), language_code, id)
            }
            // language, id
            (Some(language_code), Some(id), None) => (None, language_code, id),
            _ => return Ok(None),
        };

    let episode_no = match episode_no_unparsed {
        Some(episode_no_unparsed) => {
            // If there is an episode id, it has to be a number
            let Some(episode_no) = episode_no_unparsed.parse::<u32>().ok() else {
                return Ok(None);
            };
            Some(episode_no)
        }
        None => None,
    };

    let Some(language_code) = domain::language::LanguageCode::try_from(language_code_unparsed).ok()
    else {
        return Ok(None);
    };

    let id = match id_unparsed.is_empty() {
        true => return Ok(None),
        false => id_unparsed,
    };

    Ok(Some((
        episode_no,
        Subtitle {
            id: id.to_string(),
            language: language_code,
            path: at.as_ref().to_string_lossy().to_string(),
        },
    )))
}
