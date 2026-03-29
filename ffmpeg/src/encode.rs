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

pub async fn encode_video(
    tracks: Vec<TrackSelection>,
    output_path: impl AsRef<Path>,
) -> crate::Result<PathBuf> {
    let output_path = output_path.as_ref();

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

            let args: [Option<String>; 6] = match selection {
                TrackSelection::Video {
                    track_id, codec, ..
                } => [
                    Some("-map".to_string()),
                    Some(format!("{input_id}:{track_id}")),
                    Some(format!("-c:v:{index}")),
                    Some(codec),
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
                ],
                TrackSelection::Subtitle {
                    track_id,
                    language,
                    external_id,
                    ..
                } => [
                    Some("-map".to_string()),
                    Some(format!("{input_id}:{track_id}")),
                    language.as_ref().map(|_| format!("-metadata:s:s:{index}")),
                    language.map(|lang| format!("language={}", lang.to_iso639_2t())),
                    external_id.as_ref().map(|_| format!("-metadata:s:s:{index}")),
                    external_id.map(|ext_id| format!("EXTERNAL_ID={ext_id}")),
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
    use std::path::PathBuf;

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

    /// Copy video + audio from a simple file with no subtitles.
    #[tokio::test]
    async fn encode_no_subtitles() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_nosub.mkv");
        let output = dir.path().join("out.mkv");

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
                    duration: None
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: None,
                    language: None
                },
            ]
        );
    }

    /// Copy all tracks from a file that has a single subtitle track.
    #[tokio::test]
    async fn encode_with_one_subtitle() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("hevc_aac_1sub.mkv");
        let output = dir.path().join("out.mkv");

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
                    codec: "hevc".to_string(),
                    duration: None
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: None,
                    language: None
                },
                Track::Subtitle {
                    id: 2,
                    language: None,
                    external_id: None,
                },
            ]
        );
    }

    /// Copy all tracks from a file that carries three subtitle streams.
    /// Verifies that subtitle language tags are preserved.
    #[tokio::test]
    async fn encode_with_multiple_subtitles() {
        use domain::language::LanguageCode;

        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h265_flac_3subs.mkv");
        let output = dir.path().join("out.mkv");

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
                    codec: "hevc".to_string(),
                    duration: None
                },
                Track::Audio {
                    id: 1,
                    codec: "flac".to_string(),
                    duration: None,
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
        let video_input = fixtures.join("h264_aac_nosub.mkv");
        let audio_input = fixtures.join("hevc_aac_1sub.mkv");
        let output = dir.path().join("out.mkv");

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
                    duration: None
                },
                Track::Audio {
                    id: 1,
                    codec: "aac".to_string(),
                    duration: None,
                    language: None
                },
            ]
        );
    }

    /// Extract each of the 3 subtitle tracks from the multi-subtitle fixture
    /// into individual .srt files.
    #[tokio::test]
    async fn extract_subtitles_to_srt() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h265_flac_3subs.mkv");

        for (i, track_id) in [2usize, 3, 4].into_iter().enumerate() {
            let output = dir.path().join(format!("sub{i}.srt"));

            let result = encode_video(vec![copy_subtitle(input.clone(), track_id)], &output).await;

            assert_eq!(
                result.unwrap(),
                output,
                "subtitle track {track_id} should be saved to sub{i}.srt"
            );
        }
    }

    /// ffmpeg should fail when the output directory does not exist.
    #[tokio::test]
    async fn encode_fails_with_bad_output_path() {
        let input = fixtures_path().join("h264_aac_nosub.mkv");

        let result = encode_video(
            vec![copy_video(input.clone(), 0), copy_audio(input.clone(), 1)],
            PathBuf::from("/nonexistent_dir/output.mkv"),
        )
        .await;

        assert!(matches!(result, Err(crate::Error::NonZeroExit(_))));
    }
}
