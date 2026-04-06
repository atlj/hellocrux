use std::path::Path;

use domain::subtitles::Subtitle;
use log::{debug, error};

use crate::dir::fully_read_dir;

/// Fetch all subtitles from media path
///
/// This will read the subtitles directory, go through all available files and return a collection
/// of all subtitles.
pub async fn crawl_subtitles(
    media_path: impl AsRef<Path>,
) -> crate::crawl::Result<Vec<(Option<u32>, Subtitle)>> {
    let subtitles_dir = media_path.as_ref().join("subtitles");

    let Ok(read_dir) = fully_read_dir(&subtitles_dir).await else {
        debug!("{} doesn't exist.", subtitles_dir.display());
        return Ok(Vec::new());
    };

    let result = read_dir
        .filter(|path| domain::format::is_subtitle_file(path.path()))
        .filter_map(|subtitle_file| {
            let subtitle_path = subtitle_file.path();
            let Some((episode_no, language, id)) = parse_subtitle_file_name(&subtitle_path) else {
                error!(
                    "Subtitle at {} doesn't have a valid name",
                    subtitle_path.display()
                );
                return None;
            };

            Some((
                episode_no,
                Subtitle {
                    language,
                    id,
                    path: subtitle_file.path().to_string_lossy().to_string(),
                },
            ))
        })
        .collect();

    Ok(result)
}

/// Extract language code from subtitle path
fn parse_subtitle_file_name(
    subtitle_path: &impl AsRef<Path>,
) -> Option<(
    // Episode no
    Option<u32>,
    // Language
    domain::language::LanguageCode,
    // Id
    String,
)> {
    let file_stem = subtitle_path
        .as_ref()
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())?;

    let mut dash_iter = file_stem.split('-');
    let (episode_no_unparsed, language_code_unparsed, id_unparsed) =
        match (dash_iter.next(), dash_iter.next(), dash_iter.next()) {
            // episode_num, language, id
            (Some(episode_no), Some(language_code), Some(id)) => {
                (Some(episode_no), language_code, id)
            }
            // language, id
            (Some(language_code), Some(id), None) => (None, language_code, id),
            _ => return None,
        };

    let episode_no = match episode_no_unparsed {
        Some(episode_no_unparsed) =>
        // If there is an episode id, it has to be a number
        {
            Some(episode_no_unparsed.parse::<u32>().ok()?)
        }
        None => None,
    };

    let language_code = domain::language::LanguageCode::try_from(language_code_unparsed).ok()?;

    let id = match id_unparsed.is_empty() {
        true => return None,
        false => id_unparsed,
    };

    Some((episode_no, language_code, id.to_string()))
}
