use std::path::Path;

use domain::subtitles::Subtitle;
use log::{debug, error};

use crate::dir::fully_read_dir;

/// Fetch all subtitles from media path
///
/// This will read the subtitles directory, go through all available files and return a collection
/// of all subtitles.
pub async fn extract_subtitles(
    media_path: impl AsRef<Path>,
) -> crate::crawl::Result<Vec<Subtitle>> {
    let subtitles_dir = media_path.as_ref().join("subtitles");

    let Ok(read_dir) = fully_read_dir(&subtitles_dir).await else {
        debug!("{} doesn't exist.", subtitles_dir.display());
        return Ok(Vec::new());
    };

    let result = read_dir
        .filter(|path| domain::format::is_subtitle_file(path.path()))
        .filter_map(|subtitle_file| {
            let subtitle_path = subtitle_file.path();
            let Some(language_str) = get_language_str(&subtitle_path) else {
                error!(
                    "Subtitle at {} has no language string",
                    subtitle_path.display()
                );
                return None;
            };
            let language = domain::language::LanguageCode::try_from(language_str)
                .inspect_err(|_| {
                    error!(
                        "Subtitle at {} has an invalid language string {language_str}",
                        subtitle_path.display()
                    )
                })
                .ok()?;

            Some(Subtitle {
                language,
                path: subtitle_file.path().to_string_lossy().to_string(),
            })
        })
        .collect();

    Ok(result)
}

/// Extract language code from subtitle path
fn get_language_str(srt_path: &impl AsRef<Path>) -> Option<&str> {
    let file_stem = srt_path
        .as_ref()
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .expect("Subtitle to have valid stem");

    let (mut language_candidate, rest) = file_stem.split_once('-')?;

    if language_candidate.chars().any(|char| char.is_ascii_digit()) {
        let (second_language_candidate, _) = rest.split_once('-')?;
        language_candidate = second_language_candidate;
    }

    if language_candidate.len() != 3 {
        return None;
    }

    Some(language_candidate)
}

#[cfg(test)]
mod tests {
    use crate::crawl::subtitles::extract::get_language_str;

    #[test]
    fn test_get_language_str() {
        assert_eq!(get_language_str(&"3-eng-281971938.srt"), Some("eng"));
        assert_eq!(get_language_str(&"tur-281971938.srt"), Some("tur"));
        assert_eq!(get_language_str(&"3-eng-281971938.srt"), Some("eng"));
        assert_eq!(get_language_str(&"3-eng-281971938.srt"), Some("eng"));
    }
}
