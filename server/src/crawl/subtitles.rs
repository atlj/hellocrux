use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use domain::{language::LanguageCode, subtitles::SubtitlePath};
use log::{debug, error, info, warn};

use super::{Error, Result};

pub(super) async fn extract_subtitles(
    media_path: impl AsRef<Path>,
) -> Result<impl Iterator<Item = SubtitlePath>> {
    let subtitles_path = media_path.as_ref().join("subtitles");

    // 1. Pair all srt files with mp4 files.
    let srt_mp4_pairs = get_srt_mp4_pairs(&subtitles_path).await?;

    let mp4_futures = srt_mp4_pairs
        .into_iter()
        .filter_map(|(file_name, (srt_file, mp4_file))| {
            // Filter all entries without an srt file
            let result = srt_file.map(|srt_file| (file_name, (srt_file, mp4_file.clone())));
            if result.is_none() {
                warn!(
                    "mp4 file at {mp4_file:#?} doesn't have a corresponding srt file. Ignoring it."
                )
            }
            result
        })
        .map(|(file_name, (srt_file, mp4_file))| {
            let srt_file = srt_file.clone();
            let subtitles_path = subtitles_path.clone();

            async move {
            match mp4_file {
                // No need for a conversion
                Some(mp4_file) => Ok((file_name, (srt_file, mp4_file))),
                // mp4 is missing, we convert
                None => {
                    info!("Generating mp4 subtitle for {srt_file:#?}");
                    generate_subtitle_mp4_file(&srt_file)
                    .await
                    .map(|mp4_path| (file_name, (srt_file.clone(), mp4_path)))
                    .inspect_err(|err| {
                        error!(
                            "While extracting subtitles at {subtitles_path:#?}, couldn't convert srt file at {srt_file:#?} to mp4. Reason: {err}"
                        );
                    })},
            }
        }});

    let subtitle_results = futures::future::join_all(mp4_futures).await;

    Ok(subtitle_results
        .into_iter()
        .flatten()
        .filter_map(|(file_name, (srt_path, mp4_path))| {
            let Some(language) = get_srt_language(&file_name) else {
                error!("Ignoring subtitle at {srt_path:#?}. It doesn't include a language code in its title.");
                return None;
            };

            Some(SubtitlePath {
                language: LanguageCode::try_from(language).unwrap_or_else(|_| {
                    panic!("Unknown ISO 639-2T language code: {language}")
                }),
                srt_path: srt_path.to_string_lossy().to_string(),
                track_path: mp4_path.to_string_lossy().to_string(),
            })
        }))
}

async fn get_srt_mp4_pairs(
    subtitles_path: impl AsRef<Path>,
) -> Result<HashMap<String, (Option<PathBuf>, Option<PathBuf>)>> {
    if !tokio::fs::try_exists(&subtitles_path)
        .await
        .map_err(|_| Error::CantReadDir(subtitles_path.as_ref().into()))?
    {
        debug!("{} doesn't exist.", subtitles_path.as_ref().display());
        return Ok(HashMap::new());
    }

    let read_subtitle_dir = crate::dir::fully_read_dir(&subtitles_path)
        .await
        .map_err(|_| Error::CantReadDir(subtitles_path.as_ref().into()))?;

    let result: HashMap<String, (Option<PathBuf>, Option<PathBuf>)> =
        read_subtitle_dir.fold(HashMap::with_capacity(20), |mut map, item| {
            let path = item.path();
            let file_stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("Subtitle file to have a valid stem");
            let extension = path.extension();

            match extension.and_then(|extension| extension.to_str()) {
                Some("srt") => {
                    let entry = map.entry(file_stem.to_string()).or_default();
                    entry.0 = Some(path);
                }
                Some("mp4") => {
                    let entry = map.entry(file_stem.to_string()).or_default();
                    entry.1 = Some(path);
                }
                _ => {
                    warn!("Subtitles has non-subtitle file at {path:#?}. Ignoring it.")
                }
            };

            map
        });

    Ok(result)
}

/// Generates mp4 file at destination
async fn generate_subtitle_mp4_file(srt_path: impl AsRef<Path>) -> Result<PathBuf> {
    let mp4_path = srt_path.as_ref().with_extension("mp4");

    // 1. Check if it already exists
    if tokio::fs::try_exists(&mp4_path)
        .await
        .map_err(|_| Error::CantReadDir(mp4_path.clone()))?
    {
        warn!(
            "Tried to convert srt file at {} to mp4 but {mp4_path:#?} already exists. Skipping it.",
            srt_path.as_ref().display()
        );
        return Ok(mp4_path);
    }

    let language = get_srt_language(&srt_path).unwrap_or("eng");

    // 2. Do the conversion
    crate::ffmpeg::ffmpeg([
        // Input
        "-i",
        srt_path.as_ref().to_string_lossy().as_ref(),
        // Encode subtitles as mov_text which works with AVPlayer
        "-c:s",
        "mov_text",
        // Set language
        "-metadata:s:s:0",
        format!("language={language}").as_str(),
        // Always overwrite
        "-y",
        // Output
        mp4_path.to_string_lossy().as_ref(),
    ])
    .await?;

    // 3. Make sure the mp4 file exists
    if !tokio::fs::try_exists(&mp4_path)
        .await
        .map_err(|_| Error::CantReadDir(mp4_path.clone()))?
    {
        return Err(Error::CantConvertSubtitle(
            crate::ffmpeg::Error::MissingOutput,
        ));
    }

    Ok(mp4_path)
}

fn get_srt_language(srt_path: &impl AsRef<Path>) -> Option<&str> {
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
