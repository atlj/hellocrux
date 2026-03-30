use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub enum TrackSelection {
    Video {
        input_path: PathBuf,
        track_id: usize,
        codec: String,
    },
    Audio {
        input_path: PathBuf,
        track_id: usize,
        codec: String,
    },
    Subtitle {
        input_path: PathBuf,
        track_id: usize,
        language: Option<domain::language::LanguageCode>,
        external_id: Option<String>,
    },
}

impl TrackSelection {
    pub fn input_path(&self) -> &PathBuf {
        match self {
            TrackSelection::Video { input_path, .. } => input_path,
            TrackSelection::Audio { input_path, .. } => input_path,
            TrackSelection::Subtitle { input_path, .. } => input_path,
        }
    }
}

fn subtitle_codec_for(output_path: &Path) -> &'static str {
    match output_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp4") | Some("m4v") | Some("mov") => "mov_text",
        Some("webm") => "webvtt",
        _ => "copy",
    }
}

pub async fn encode_video(
    tracks: Vec<TrackSelection>,
    output_path: impl AsRef<Path>,
) -> crate::Result<PathBuf> {
    let output_path = output_path.as_ref();
    let subtitle_codec = subtitle_codec_for(output_path);

    let deduped_inputs = tracks
        .iter()
        .map(|track| track.input_path().clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let input_args = deduped_inputs
        .iter()
        .flat_map(|track| ["-i".to_string(), track.to_string_lossy().to_string()]);

    let mapping_args = tracks
        .into_iter()
        .scan(
            (0_usize, 0_usize, 0_usize),
            |(video_index, audio_index, subtitle_index),
             track|
             -> Option<(usize, crate::TrackSelection)> {
                match &track {
                    TrackSelection::Video { .. } => {
                        *video_index += 1;
                        Some((*video_index - 1, track))
                    }
                    TrackSelection::Audio { .. } => {
                        *audio_index += 1;
                        Some((*audio_index - 1, track))
                    }
                    TrackSelection::Subtitle { .. } => {
                        *subtitle_index += 1;
                        Some((*subtitle_index - 1, track))
                    }
                }
            },
        )
        .flat_map(|(index, selection)| {
            let input_path = selection.input_path();
            let input_id = deduped_inputs
                .iter()
                .enumerate()
                .find_map(|(index, input_param)| {
                    if input_param == input_path {
                        Some(index)
                    } else {
                        None
                    }
                })
                .expect("Input path to be a part of deduped inputs");

            let args: [Option<String>; 8] = match selection {
                TrackSelection::Video {
                    track_id, codec, ..
                } => [
                    Some("-map".to_string()),
                    Some(format!("{input_id}:{track_id}")),
                    Some(format!("-c:v:{index}")),
                    Some(codec),
                    None,
                    None,
                    None,
                    None,
                ],
                TrackSelection::Audio {
                    track_id, codec, ..
                } => [
                    Some("-map".to_string()),
                    Some(format!("{input_id}:{track_id}")),
                    Some(format!("-c:a:{index}")),
                    Some(codec),
                    None,
                    None,
                    None,
                    None,
                ],
                TrackSelection::Subtitle {
                    track_id,
                    language,
                    external_id,
                    ..
                } => [
                    Some("-map".to_string()),
                    Some(format!("{input_id}:{track_id}")),
                    Some(format!("-c:s:{index}")),
                    Some(subtitle_codec.to_string()),
                    language.as_ref().map(|_| format!("-metadata:s:s:{index}")),
                    language.map(|lang| format!("language={}", lang.to_iso639_2t())),
                    external_id.as_ref().map(|_| format!("-metadata:s:s:{index}")),
                    external_id.map(|ext_id| format!("handler_name={ext_id}")),
                ],
            };
            args.into_iter().flatten()
        });

    let args = input_args
        .chain(mapping_args)
        .chain([output_path.to_string_lossy().to_string()]);

    crate::spawn::ffmpeg(args).await?;

    // Make sure output exists now
    if let Ok(true) = tokio::fs::try_exists(output_path).await {
        return Ok(output_path.to_path_buf());
    }

    Err(crate::Error::MissingOutput)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use domain::language::LanguageCode;

    use super::{TrackSelection, encode_video};
    use crate::{Track, get_tracks};

    fn fixtures_path() -> PathBuf {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into()
    }

    fn copy_video(input_path: PathBuf, track_id: usize) -> TrackSelection {
        TrackSelection::Video {
            input_path,
            track_id,
            codec: "copy".to_string(),
        }
    }

    fn copy_audio(input_path: PathBuf, track_id: usize) -> TrackSelection {
        TrackSelection::Audio {
            input_path,
            track_id,
            codec: "copy".to_string(),
        }
    }

    fn copy_subtitle(input_path: PathBuf, track_id: usize) -> TrackSelection {
        TrackSelection::Subtitle {
            input_path,
            track_id,
            language: None,
            external_id: None,
        }
    }

    async fn tracks_of(path: &PathBuf) -> Vec<Track> {
        let mut tracks = get_tracks(path)
            .await
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        tracks.sort_by_key(|t| *t.id());
        tracks
    }

    /// Copy video + audio from an MP4 with no subtitles.
    #[tokio::test]
    async fn encode_no_subtitles() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_nosub.mp4");
        let output = dir.path().join("out.mp4");

        let result = encode_video(
            vec![copy_video(input.clone(), 0), copy_audio(input.clone(), 1)],
            &output,
        )
        .await;

        assert_eq!(result.unwrap(), output);
        assert_eq!(
            tracks_of(&output).await,
            vec![
                Track::Video {
                    id: 0,
                    codec: "h264".to_string(),
                    duration: Some(Duration::from_secs(2))
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: Some(Duration::from_secs(2)),
                    language: None
                },
            ]
        );
    }

    /// Copy all tracks from an MP4 with one subtitle track.
    #[tokio::test]
    async fn encode_with_one_subtitle() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_1sub.mp4");
        let output = dir.path().join("out.mp4");

        encode_video(
            vec![
                copy_video(input.clone(), 0),
                copy_audio(input.clone(), 1),
                copy_subtitle(input.clone(), 2),
            ],
            &output,
        )
        .await
        .unwrap();

        assert_eq!(
            tracks_of(&output).await,
            vec![
                Track::Video {
                    id: 0,
                    codec: "h264".to_string(),
                    duration: Some(Duration::from_secs(2))
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: Some(Duration::from_secs(2)),
                    language: None
                },
                Track::Subtitle {
                    id: 2,
                    language: Some(LanguageCode::English),
                    external_id: None,
                },
            ]
        );
    }

    /// Copy all tracks from an MP4 with three subtitle streams.
    /// Verifies language tags and handler_name-based external_id are preserved.
    #[tokio::test]
    async fn encode_with_multiple_subtitles() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_3subs.mp4");
        let output = dir.path().join("out.mp4");

        encode_video(
            vec![
                copy_video(input.clone(), 0),
                copy_audio(input.clone(), 1),
                copy_subtitle(input.clone(), 2),
                copy_subtitle(input.clone(), 3),
                copy_subtitle(input.clone(), 4),
            ],
            &output,
        )
        .await
        .unwrap();

        assert_eq!(
            tracks_of(&output).await,
            vec![
                Track::Video {
                    id: 0,
                    codec: "h264".to_string(),
                    duration: Some(Duration::from_secs(2))
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: Some(Duration::from_secs(2)),
                    language: None
                },
                Track::Subtitle {
                    id: 2,
                    language: Some(LanguageCode::English),
                    external_id: Some("sub_ext_001".to_string()),
                },
                Track::Subtitle {
                    id: 3,
                    language: Some(LanguageCode::French),
                    external_id: Some("sub_ext_002".to_string()),
                },
                Track::Subtitle {
                    id: 4,
                    language: None,
                    external_id: Some("sub_ext_003".to_string()),
                },
            ]
        );
    }

    /// Mix video from one file and audio from another (two distinct -i inputs).
    #[tokio::test]
    async fn encode_multi_input() {
        let dir = tempfile::tempdir().unwrap();
        let fixtures = fixtures_path();
        let video_input = fixtures.join("h264_aac_nosub.mp4");
        let audio_input = fixtures.join("h264_aac_1sub.mp4");
        let output = dir.path().join("out.mp4");

        encode_video(
            vec![
                copy_video(video_input.clone(), 0),
                copy_audio(audio_input.clone(), 1),
            ],
            &output,
        )
        .await
        .unwrap();

        assert_eq!(
            tracks_of(&output).await,
            vec![
                Track::Video {
                    id: 0,
                    codec: "h264".to_string(),
                    duration: Some(Duration::from_secs(2))
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: Some(Duration::from_secs(2)),
                    language: None
                },
            ]
        );
    }

    /// Embed external SRT subtitles with language and external_id into an MP4.
    /// Verifies the subrip→mov_text transcode path and handler_name round-trip.
    #[tokio::test]
    async fn encode_external_subtitles() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_nosub.mp4");
        let output = dir.path().join("out.mp4");

        let sub_eng = dir.path().join("sub_eng.srt");
        let sub_fra = dir.path().join("sub_fra.srt");
        tokio::fs::write(&sub_eng, b"1\n00:00:00,000 --> 00:00:02,000\nHello\n")
            .await
            .unwrap();
        tokio::fs::write(&sub_fra, b"1\n00:00:00,000 --> 00:00:02,000\nBonjour\n")
            .await
            .unwrap();

        encode_video(
            vec![
                copy_video(input.clone(), 0),
                copy_audio(input.clone(), 1),
                TrackSelection::Subtitle {
                    input_path: sub_eng.clone(),
                    track_id: 0,
                    language: Some(LanguageCode::English),
                    external_id: Some("ext_eng_001".to_string()),
                },
                TrackSelection::Subtitle {
                    input_path: sub_fra.clone(),
                    track_id: 0,
                    language: Some(LanguageCode::French),
                    external_id: Some("ext_fra_002".to_string()),
                },
            ],
            &output,
        )
        .await
        .unwrap();

        assert_eq!(
            tracks_of(&output).await,
            vec![
                Track::Video {
                    id: 0,
                    codec: "h264".to_string(),
                    duration: Some(Duration::from_secs(2))
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: Some(Duration::from_secs(2)),
                    language: None
                },
                Track::Subtitle {
                    id: 2,
                    language: Some(LanguageCode::English),
                    external_id: Some("ext_eng_001".to_string()),
                },
                Track::Subtitle {
                    id: 3,
                    language: Some(LanguageCode::French),
                    external_id: Some("ext_fra_002".to_string()),
                },
            ]
        );
    }

    /// ffmpeg should fail when the output directory does not exist.
    #[tokio::test]
    async fn encode_fails_with_bad_output_path() {
        let input = fixtures_path().join("h264_aac_nosub.mp4");

        let result = encode_video(
            vec![copy_video(input.clone(), 0), copy_audio(input.clone(), 1)],
            PathBuf::from("/nonexistent_dir/output.mp4"),
        )
        .await;

        assert!(matches!(result, Err(crate::Error::NonZeroExit(_))));
    }
}

