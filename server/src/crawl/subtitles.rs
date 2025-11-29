use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{Error, Result};
use domain::{language::LanguageCode, subtitles::Subtitle};
use log::{info, warn};

// TODO reduce duplication between this and series subtitles
pub async fn try_generate_movie_subtitles(path: impl AsRef<Path>) -> Result<Vec<Subtitle>> {
    let explored_subtitles = explore_subtitles(&path).await?;

    let mp4_subtitles_features = explored_subtitles
        .iter()
        .flat_map(
            |(path, (explored_subtitle, mp4_exists))| match explored_subtitle {
                Some(subtitle) if !mp4_exists => Some((path, subtitle)),
                _ => None,
            },
        )
        .map(|(path, explored_subtitle)| async {
            info!("Generating mp4 subtitle for {}", path.display());
            generate_subtitle_mp4(path.with_extension(&explored_subtitle.3), explored_subtitle)
                .await
        });

    let mp4_subtitle_generation_results = futures::future::join_all(mp4_subtitles_features).await;
    if let Some(err_result) = mp4_subtitle_generation_results
        .into_iter()
        .find(|result| result.is_err())
    {
        err_result?;
    }

    let result: Vec<Subtitle> = explored_subtitles
        .into_iter()
        .flat_map(|(path, (explored_subtitle, _))| match explored_subtitle {
            Some(explored_subtitle) => Some(Subtitle {
                name: explored_subtitle.0,
                language_iso639_2t: explored_subtitle.1.to_iso639_2t().to_string(),
                path: path
                    .with_extension(explored_subtitle.3)
                    .to_string_lossy()
                    .to_string(),
                track_path: path.with_extension("mp4").to_string_lossy().to_string(),
            }),
            None => {
                warn!(
                    "An mp4 subtitle found but there is no text file for it. Ignoring it. Path: {}",
                    path.display()
                );
                None
            }
        })
        .collect();

    Ok(result)
}

pub async fn try_generate_series_subtitles(
    path: impl AsRef<Path>,
) -> Result<HashMap<usize, Vec<Subtitle>>> {
    let explored_subtitles = explore_subtitles(&path).await?;

    let mp4_subtitles_features = explored_subtitles
        .iter()
        .flat_map(
            |(path, (explored_subtitle, mp4_exists))| match explored_subtitle {
                Some(subtitle) if !mp4_exists => Some((path, subtitle)),
                _ => None,
            },
        )
        .map(|(path, explored_subtitle)| async {
            info!("Generating mp4 subtitle for {}", path.display());
            generate_subtitle_mp4(path.with_extension(&explored_subtitle.3), explored_subtitle)
                .await
        });

    let mp4_subtitle_generation_results = futures::future::join_all(mp4_subtitles_features).await;
    if let Some(err_result) = mp4_subtitle_generation_results
        .into_iter()
        .find(|result| result.is_err())
    {
        err_result?;
    }

    let result: HashMap<usize, Vec<Subtitle>> = explored_subtitles
        .into_iter()
        .fold(HashMap::new(), |mut map, (path, (explored_subtitle, _))| {
            let explored_subtitle = if let Some(subs) = explored_subtitle {subs} else {
                warn!(
                    "An mp4 subtitle found but there is no text file for it. Ignoring it. Path: {}",
                    path.display()
                );

                return map
            };

            let episode_number = if let Some(num) = explored_subtitle.2 {
                num
            } else {
                warn!("A subtitle at {} was crawled for a series but it doesn't have an episode number. Ignoring it.", path.display());

                return map
            };

            let subs_vec = map.entry(episode_number).or_default();

            subs_vec.push(Subtitle {
                name: explored_subtitle.0,
                language_iso639_2t: explored_subtitle.1.to_iso639_2t().to_string(),
                path: path.with_extension(explored_subtitle.3).to_string_lossy().to_string(),
                track_path: path.with_extension("mp4").to_string_lossy().to_string()
            });

            map
        });

    Ok(result)
}

async fn explore_subtitles(
    path: impl AsRef<Path>,
) -> Result<HashMap<PathBuf, (Option<ExploredSubtitle>, bool)>> {
    let dir_entries = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let mapping = dir_entries.fold(HashMap::new(), |mut map, entry| {
        let path = entry.path();
        let explored_subtitle = if let Some(parsed) = parse_subtitle_name(&path) {
            parsed
        } else {
            return map;
        };
        let extension =
            if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
                extension
            } else {
                return map;
            };

        if !matches!(extension, "srt" | "vtt" | "mp4") {
            return map;
        }

        {
            let entry = map.entry(path.with_extension("")).or_insert((None, false));
            match extension {
                "mp4" => entry.1 = true,
                "srt" | "vtt" => entry.0 = Some(explored_subtitle),
                _ => unreachable!(
                    "Non supported extensions should've been eleminated. Extension: {extension}"
                ),
            }
        }

        map
    });

    Ok(mapping)
}

type ExploredSubtitle = (String, LanguageCode, Option<usize>, String);

fn parse_subtitle_name(path: impl AsRef<Path>) -> Option<ExploredSubtitle> {
    let file_stem = path.as_ref().file_stem()?.to_str()?;
    let episode_no = file_stem
        .chars()
        .map_while(|char| char.to_digit(10).map(|val| val as usize))
        .fold(None, |acc, digit| match acc {
            Some(number) => Some(number * 10 + digit),
            None => Some(digit),
        });

    let language_code = {
        let start_index = file_stem.find(|char: char| !char.is_ascii_digit())?;
        file_stem
            .get(start_index..start_index + 3)?
            .try_into()
            .ok()?
    };

    let name = file_stem
        .chars()
        .skip_while(|char| char.is_ascii_digit())
        .skip(3)
        .collect::<String>();

    let extension = path.as_ref().extension()?.to_str()?;

    Some((name, language_code, episode_no, extension.to_string()))
}

async fn generate_subtitle_mp4(
    path: impl AsRef<Path>,
    explored_subtitle: &ExploredSubtitle,
) -> Result<()> {
    crate::ffmpeg::ffmpeg([
        // Input
        "-i",
        path.as_ref().to_string_lossy().as_ref(),
        // Encode subtitles as mov_text which works with AVPlayer
        "-c:s",
        "mov_text",
        // Set language
        "-metadata:s:s:0",
        format!("language={}", explored_subtitle.1.to_iso639_1()).as_str(),
        // Always overwrite
        "-y",
        // Output
        // TODO: perhaps remove this to string abomination
        path.as_ref()
            .with_extension("mp4")
            .to_string_lossy()
            .as_ref(),
    ])
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use domain::language::LanguageCode;

    use crate::crawl::subtitles::{
        explore_subtitles, generate_subtitle_mp4, parse_subtitle_name,
        try_generate_movie_subtitles, try_generate_series_subtitles,
    };

    #[tokio::test]
    async fn generate_movie_subtitles() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/crawl/subtitles/movie")).await;
        tokio::fs::create_dir_all(test_data_path.join("tmp/crawl/subtitles/movie"))
            .await
            .unwrap();
        copy_dir_all(
            test_data_path.join("crawl/generate_movie_subs"),
            test_data_path.join("tmp/crawl/subtitles/movie"),
        )
        .unwrap();

        let result = try_generate_movie_subtitles(test_data_path.join("tmp/crawl/subtitles/movie"))
            .await
            .unwrap();

        assert!(result.len() == 2);

        assert!(
            tokio::fs::try_exists(
                test_data_path.join("tmp/crawl/subtitles/movie/engexample_subs.mp4")
            )
            .await
            .unwrap()
        );
    }

    #[tokio::test]
    async fn generate_series_subtitles() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/crawl/subtitles/series")).await;
        tokio::fs::create_dir_all(test_data_path.join("tmp/crawl/subtitles/series"))
            .await
            .unwrap();
        copy_dir_all(
            test_data_path.join("crawl/generate_series_subs"),
            test_data_path.join("tmp/crawl/subtitles/series"),
        )
        .unwrap();

        let result =
            try_generate_series_subtitles(test_data_path.join("tmp/crawl/subtitles/series"))
                .await
                .unwrap();

        let first_episode = result.get(&1).unwrap();
        assert!(first_episode.len() == 1);
        let first = first_episode.first().unwrap();
        assert!(first.language_iso639_2t == LanguageCode::Turkish.to_iso639_2t());
        assert_eq!(first.name, "example_subs");

        let second_episode = result.get(&2).unwrap();
        assert!(second_episode.len() == 1);
        let second = second_episode.first().unwrap();
        assert!(second.language_iso639_2t == LanguageCode::English.to_iso639_2t());
        assert_eq!(second.name, "example_subs");

        assert!(result.len() == 2);
        assert!(
            tokio::fs::try_exists(
                test_data_path.join("tmp/crawl/subtitles/series/2engexample_subs.mp4")
            )
            .await
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_generate_mp4_subtitle() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/crawl/mp4_subs")).await;
        tokio::fs::create_dir_all(test_data_path.join("tmp/crawl/mp4_subs"))
            .await
            .unwrap();

        let path = test_data_path.join("crawl/subs");
        generate_subtitle_mp4(
            path.join("turexample_subs.vtt"),
            &(
                "example_subs".to_string(),
                LanguageCode::Turkish,
                None,
                "vtt".to_string(),
            ),
        )
        .await
        .unwrap();

        assert!(
            tokio::fs::try_exists(path.join("turexample_subs.mp4"))
                .await
                .unwrap()
        );

        tokio::fs::remove_file(path.join("turexample_subs.mp4"))
            .await
            .unwrap();
        generate_subtitle_mp4(
            path.join("turexample_subs.srt"),
            &(
                "example_subs".to_string(),
                LanguageCode::Turkish,
                None,
                "srt".to_string(),
            ),
        )
        .await
        .unwrap();
        assert!(
            tokio::fs::try_exists(path.join("turexample_subs.mp4"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn subtitle_pairs() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/explore_subtitles");
        let result = explore_subtitles(&path).await.unwrap();
        let values: Box<[_]> = result.into_values().collect();

        assert!(values.contains(&(
            Some((
                "heyyy".to_string(),
                LanguageCode::English,
                Some(2),
                "vtt".to_string()
            )),
            false,
        ),));
        assert!(values.contains(&(
            Some((
                "hey".to_string(),
                LanguageCode::Turkish,
                Some(1),
                "srt".to_string()
            )),
            true
        )));
        assert!(values.contains(&(
            Some((
                "nope".to_string(),
                LanguageCode::English,
                None,
                "srt".to_string()
            )),
            false
        )));
    }

    #[test]
    fn subtitle_name() {
        assert_eq!(
            parse_subtitle_name("0231enghey.srt").unwrap(),
            (
                "hey".to_string(),
                LanguageCode::English,
                Some(231),
                "srt".to_string()
            )
        );
        assert_eq!(
            parse_subtitle_name("enghey.srt").unwrap(),
            (
                "hey".to_string(),
                LanguageCode::English,
                None,
                "srt".to_string()
            )
        );
        assert!(parse_subtitle_name("a.srt").is_none());
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
