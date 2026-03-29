use std::{collections::HashSet, path::PathBuf};

#[derive(Debug, Clone)]
pub struct EncodeOptions {
    pub video_tracks: Vec<TrackSelection>,
    pub audio_tracks: Vec<TrackSelection>,
    pub subtitle_tracks: Vec<TrackSelection>,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TrackSelection {
    pub input_path: PathBuf,
    pub track_id: usize,
    pub codec: String,
}

pub async fn encode_video(
    EncodeOptions {
        video_tracks,
        audio_tracks,
        subtitle_tracks,
        output_path,
    }: EncodeOptions,
) -> crate::Result<PathBuf> {
    let track_iter = video_tracks
        .iter()
        .chain(audio_tracks.iter().chain(subtitle_tracks.iter()));

    let deduped_tracks = track_iter
        .map(|track| track.input_path.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let input_args = deduped_tracks
        .iter()
        .flat_map(|track| ["-i".to_string(), track.to_string_lossy().to_string()]);

    let video_mapping_args = video_tracks
        .into_iter()
        .enumerate()
        .flat_map(|(index, track)| {
            let input_id = deduped_tracks
                .iter()
                .enumerate()
                .find_map(|(index, deduped_path)| {
                    if deduped_path == &track.input_path {
                        Some(index)
                    } else {
                        None
                    }
                })
                .expect("Input's track should be already included in deduped tracks");

            [
                "-map".to_string(),
                format!("{input_id}:{}", track.track_id),
                format!("-c:v:{index}"),
                track.codec,
            ]
        });

    let audio_mapping_args = audio_tracks
        .into_iter()
        .enumerate()
        .flat_map(|(index, track)| {
            let input_id = deduped_tracks
                .iter()
                .enumerate()
                .find_map(|(index, deduped_path)| {
                    if deduped_path == &track.input_path {
                        Some(index)
                    } else {
                        None
                    }
                })
                .expect("Input's track should be already included in deduped tracks");

            [
                "-map".to_string(),
                format!("{input_id}:{}", track.track_id),
                format!("-c:a:{index}"),
                track.codec,
            ]
        });

    let subtitle_mapping_args =
        subtitle_tracks
            .into_iter()
            .enumerate()
            .flat_map(|(index, track)| {
                let input_id = deduped_tracks
                    .iter()
                    .enumerate()
                    .find_map(|(index, deduped_path)| {
                        if deduped_path == &track.input_path {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .expect("Input's track should be already included in deduped tracks");

                [
                    "-map".to_string(),
                    format!("{input_id}:{}", track.track_id),
                    format!("-c:s:{index}"),
                    track.codec,
                ]
            });

    let args = input_args
        .chain(video_mapping_args)
        .chain(audio_mapping_args)
        .chain(subtitle_mapping_args)
        .chain([output_path.to_string_lossy().to_string()]);

    crate::spawn::ffmpeg(args).await?;

    // Make sure output exists now
    if let Ok(true) = tokio::fs::try_exists(&output_path).await {
        return Ok(output_path);
    }

    Err(crate::Error::MissingOutput)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{EncodeOptions, TrackSelection, encode_video};
    use crate::{Track, get_tracks};

    fn fixtures_path() -> PathBuf {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into()
    }

    fn copy(input_path: PathBuf, track_id: usize) -> TrackSelection {
        TrackSelection {
            input_path,
            track_id,
            codec: "copy".to_string(),
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

        let result = encode_video(EncodeOptions {
            video_tracks: vec![copy(input.clone(), 0)],
            audio_tracks: vec![copy(input.clone(), 1)],
            subtitle_tracks: vec![],
            output_path: output.clone(),
        })
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

        encode_video(EncodeOptions {
            video_tracks: vec![copy(input.clone(), 0)],
            audio_tracks: vec![copy(input.clone(), 1)],
            subtitle_tracks: vec![copy(input.clone(), 2)],
            output_path: output.clone(),
        })
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
                    external_id: None
                },
            ]
        );
    }

    /// Copy all tracks from a file that carries three subtitle streams.
    /// Verifies that subtitle language tags and external IDs are preserved.
    #[tokio::test]
    async fn encode_with_multiple_subtitles() {
        use domain::language::LanguageCode;

        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h265_flac_3subs.mkv");
        let output = dir.path().join("out.mkv");

        encode_video(EncodeOptions {
            video_tracks: vec![copy(input.clone(), 0)],
            audio_tracks: vec![copy(input.clone(), 1)],
            subtitle_tracks: vec![
                copy(input.clone(), 2),
                copy(input.clone(), 3),
                copy(input.clone(), 4),
            ],
            output_path: output.clone(),
        })
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

        encode_video(EncodeOptions {
            video_tracks: vec![copy(video_input.clone(), 0)],
            audio_tracks: vec![copy(audio_input.clone(), 1)],
            subtitle_tracks: vec![],
            output_path: output.clone(),
        })
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

    /// ffmpeg should fail when the output directory does not exist.
    #[tokio::test]
    async fn encode_fails_with_bad_output_path() {
        let input = fixtures_path().join("h264_aac_nosub.mkv");
        let output = PathBuf::from("/nonexistent_dir/output.mkv");

        let result = encode_video(EncodeOptions {
            video_tracks: vec![copy(input.clone(), 0)],
            audio_tracks: vec![copy(input.clone(), 1)],
            subtitle_tracks: vec![],
            output_path: output,
        })
        .await;

        assert!(matches!(result, Err(crate::Error::NonZeroExit(_))));
    }
}
