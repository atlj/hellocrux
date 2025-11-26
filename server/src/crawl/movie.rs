use domain::{LanguageCode, Subtitle};

use super::{Error, Result};
use std::path::Path;

pub(super) async fn try_extract_movie_paths(
    path: impl AsRef<Path>,
) -> Result<Option<domain::MediaPaths>> {
    let mut read_dir = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let media = read_dir.find_map(|entry| {
        let path = entry.path();
        if !domain::format::is_supported_video_file(&path) {
            return None;
        }

        Some(path.to_string_lossy().to_string())
    });

    let subtitles = {
        let path = path.as_ref().join("subtitles");
        if tokio::fs::try_exists(&path)
            .await
            .map_err(|_| super::Error::CantReadDir(path.clone()))?
        {
            Some(try_extract_subtitles(&path).await?)
        } else {
            None
        }
    }
    .unwrap_or(Box::new([]));

    Ok(media.map(|media| domain::MediaPaths { media, subtitles }))
}

async fn try_extract_subtitles(path: impl AsRef<Path>) -> Result<Box<[Subtitle]>> {
    let dir_contents = crate::dir::fully_read_dir(&path)
        .await
        .map_err(|_| Error::CantReadDir(path.as_ref().into()))?;

    let result = dir_contents
        .flat_map(|entry| {
            let path = entry.path();

            if path.is_dir() {
                return None;
            }

            if let Some(extension) = path.extension() {
                if extension == "mp4" {
                    return Some(path);
                }
            }

            None
        })
        .flat_map(|path| {
            parse_subtitle_name(&path)
                .map(|(language, name)| (path.to_string_lossy().to_string(), language, name))
        })
        .map(|(path, language, name)| Subtitle {
            name,
            language_iso639_2t: language.to_iso639_2t().to_string(),
            path,
        })
        .collect();

    Ok(result)
}

fn parse_subtitle_name(path: impl AsRef<Path>) -> Option<(LanguageCode, String)> {
    let file_stem = path.as_ref().file_stem()?.to_str()?;
    let language_code: LanguageCode = file_stem.get(0..3)?.try_into().ok()?;
    let name: String = file_stem.chars().skip(3).collect();

    Some((language_code, name))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use domain::LanguageCode;

    use crate::crawl::movie::{parse_subtitle_name, try_extract_movie_paths};

    #[tokio::test]
    async fn extract_movie_path() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        let path = test_data_path.join("crawl/example_movie");
        let result = try_extract_movie_paths(&path).await.unwrap().unwrap();
        assert!(result.media.contains("hey.mp4"));
        let subtitles = result.subtitles.first().unwrap();
        assert!(subtitles.path.contains("engSubs.mp4"));
        assert_eq!(subtitles.language_iso639_2t, "eng");
        assert_eq!(subtitles.name, "Subs");
    }

    #[test]
    fn subtitle_name() {
        assert_eq!(
            parse_subtitle_name("turSubtitlesx265.vtt").unwrap(),
            (LanguageCode::Turkish, "Subtitlesx265".to_string())
        );
    }
}
