use domain::{LanguageCode, SeriesContents, Subtitle};

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
        .try_fold(HashMap::new(), |mut map, val| match val {
            Ok((season_no, season_contents)) => {
                map.insert(season_no, season_contents);
                Ok(map)
            }
            Err(err) => Err(err),
        })?;

    Ok(Some(result))
}

async fn try_extract_season(path: impl AsRef<Path>) -> Result<Option<domain::SeasonContents>> {
    let read_dir = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let mut subtitles = {
        let path = path.as_ref().join("subtitles");
        if tokio::fs::try_exists(&path)
            .await
            .map_err(|_| super::Error::CantReadDir(path.clone()))?
        {
            Some(try_extract_subtitles(&path).await?)
        } else {
            None
        }
    };

    let result: domain::SeasonContents = read_dir.fold(HashMap::new(), |mut map, entry| {
        let current_path = entry.path();
        if !domain::format::is_supported_video_file(&current_path) {
            return map;
        }

        if let Some(episode_no) =
            super::get_numeric_content(entry.file_name().to_string_lossy().as_ref())
        {
            let subtitles = subtitles
                .as_mut()
                .and_then(|subtitles| {
                    subtitles
                        .remove(&(episode_no as usize))
                        .map(|subtitles| subtitles.into_boxed_slice())
                })
                .unwrap_or(Box::new([]));

            let media_paths = domain::MediaPaths {
                subtitles,
                media: current_path.to_string_lossy().into(),
            };

            map.insert(episode_no, media_paths);
        }

        map
    });

    if result.is_empty() {
        return Ok(None);
    }

    Ok(Some(result))
}

async fn try_extract_subtitles(
    path: impl AsRef<Path>,
) -> super::Result<HashMap<usize, Vec<Subtitle>>> {
    let dir_contents = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let result = dir_contents
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return None;
            }

            if let Some(extension) = path.extension() {
                if extension == "srt" {
                    return Some(path);
                }
            }

            None
        })
        .flat_map(|path| {
            parse_subtitle_name(&path)
                .map(|(episode_no, language, name)| (path, episode_no, language, name))
        })
        .fold(
            HashMap::new(),
            |mut map: HashMap<usize, Vec<Subtitle>>, (path, episode_no, language, name)| {
                let subtitle = Subtitle {
                    path: path.to_string_lossy().to_string(),
                    name,
                    language,
                };

                if let Some(entry) = map.get_mut(&episode_no) {
                    entry.push(subtitle);
                } else {
                    map.insert(episode_no, vec![subtitle]);
                }

                map
            },
        );

    Ok(result)
}

fn parse_subtitle_name(path: impl AsRef<Path>) -> Option<(usize, LanguageCode, String)> {
    let file_stem = path.as_ref().file_stem()?.to_str()?;
    let episode_no = file_stem.chars().map_while(|char| char.to_digit(10)).fold(
        None,
        |acc, digit| match acc {
            Some(number) => Some(number * 10 + digit),
            None => Some(digit),
        },
    )? as usize;
    let language_code = {
        let start_index = file_stem.find(|char: char| !char.is_ascii_digit())?;
        file_stem
            .get(start_index..start_index + 2)?
            .try_into()
            .ok()?
    };
    let name = file_stem
        .chars()
        .skip_while(|char| char.is_ascii_digit())
        .skip(2)
        .collect::<String>();
    Some((episode_no, language_code, name))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::crawl::series::{try_extract_season, try_extract_series};

    #[tokio::test]
    async fn extract_series() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/example_series");
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
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/example_series");

        assert!(
            try_extract_season(path.join("1"))
                .await
                .unwrap()
                .unwrap()
                .get(&1)
                .unwrap()
                .media
                .contains("1.mp4")
        );
    }
}
