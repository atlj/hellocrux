use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{Error, Result};
use domain::{LanguageCode, Subtitle};

async fn try_generate_movie_subtitles(path: impl AsRef<Path>) -> Result<Option<Subtitle>> {
    todo!()
}

async fn generate_subtitle_mp4(
    path: impl AsRef<Path>,
    explored_subtitle: ExploredSubtitle,
) -> Result<()> {
    crate::ffmpeg::ffmpeg([
        // Input
        "-i",
        path.as_ref().to_string_lossy().to_string().as_str(),
        // Encode subtitles as mov_text which works with AVPlayer
        "-c:s",
        "mov_text",
        // Set language
        "-metadata:s:s:0",
        // Always overwrite
        "-y",
        format!("language={}", explored_subtitle.1.to_iso639_2t()).as_str(),
        // Output
        path.as_ref()
            .with_extension("mp4")
            .to_string_lossy()
            .to_string()
            .as_str(),
    ])
    .await?;

    Ok(())
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
        let file_stem = path.file_stem().unwrap_or_else(|| {
            panic!("Found a language code but there was no file stem at {path:#?}")
        });
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
            let entry = map.entry(file_stem.into()).or_insert((None, false));
            match extension {
                "mp4" => entry.1 = true,
                "srt" | "vtt" => entry.0 = Some(explored_subtitle),
                _ => unreachable!(
                    "Non supported extensions should've been eliminated. Extension: {extension}"
                ),
            }
        }

        map
    });

    Ok(mapping)
}

type ExploredSubtitle = (String, LanguageCode, Option<usize>);

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

    Some((name, language_code, episode_no))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use domain::LanguageCode;

    use crate::crawl::subtitles::{explore_subtitles, parse_subtitle_name};

    #[tokio::test]
    async fn subtitle_pairs() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/explore_subtitles");
        let result = explore_subtitles(&path).await.unwrap();
        let values: Box<[_]> = result.into_values().collect();

        assert!(values.contains(&(
            Some(("heyyy".to_string(), LanguageCode::English, Some(2,),)),
            false,
        ),));
        assert!(values.contains(&(
            Some(("hey".to_string(), LanguageCode::Turkish, Some(1))),
            true
        )));
        assert!(values.contains(&(
            Some(("nope".to_string(), LanguageCode::English, None)),
            false
        )));
    }

    #[test]
    fn subtitle_name() {
        assert_eq!(
            parse_subtitle_name("0231enghey.srt").unwrap(),
            ("hey".to_string(), LanguageCode::English, Some(231))
        );
        assert_eq!(
            parse_subtitle_name("enghey.srt").unwrap(),
            ("hey".to_string(), LanguageCode::English, None)
        );
        assert!(parse_subtitle_name("a.srt").is_none());
    }
}
