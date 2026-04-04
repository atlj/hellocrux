use std::path::Path;

use crate::spawn::ffprobe;

pub async fn get_tracks(
    media_file: impl AsRef<Path>,
) -> crate::Result<impl Iterator<Item = crate::Result<domain::Track>>> {
    let result_string = ffprobe([
        // Error on empty
        "-v",
        "error",
        // Output JSON
        "-print_format",
        "json",
        // Only show relevant entries
        "-show_entries",
        "stream=index,codec_name,codec_type,duration:stream_tags=language,handler_name",
        &media_file.as_ref().as_os_str().to_string_lossy(),
    ])
    .await?;

    let parsed = serde_json::from_str::<dto::FfprobeOutput>(&result_string)
        .map_err(|_| crate::Error::UnexpectedOutput(result_string))?;

    Ok(parsed
        .streams
        .into_iter()
        .map(dto::FfprobeStream::try_into)
        .map(|result| result.map_err(dto::Error::into)))
}

pub mod dto {
    use std::time::Duration;

    #[derive(Debug, thiserror::Error, PartialEq, Eq)]
    pub enum Error {
        #[error("Track has an unknown codec type '{0}'")]
        UnknownCodecType(String),
        #[error("Track has no codec {0:#?}")]
        NoCodec(FfprobeStream),
        #[error("Couldn't parse track's duration {0:#?}")]
        CouldntParseDuration(FfprobeStream),
        #[error("Track has no duration {0:#?}")]
        NoDuration(FfprobeStream),
    }

    impl From<Error> for crate::Error {
        fn from(val: Error) -> Self {
            crate::Error::CouldntGetTracks(val)
        }
    }

    #[derive(Debug, serde::Deserialize, Clone)]
    pub struct FfprobeOutput {
        pub streams: Vec<FfprobeStream>,
    }

    #[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
    pub struct FfprobeStream {
        pub index: usize,
        pub codec_name: Option<String>,
        pub codec_type: String,
        pub duration: Option<String>,
        pub tags: Option<FfprobeStreamTags>,
    }

    impl TryInto<domain::Track> for FfprobeStream {
        type Error = Error;

        fn try_into(self) -> Result<domain::Track, Self::Error> {
            let duration = match self.duration.as_ref() {
                Some(duration_string) => {
                    let seconds = duration_string
                        .parse::<f64>()
                        .map_err(|_| Error::CouldntParseDuration(self.clone()))?;
                    Some(Duration::from_secs_f64(seconds))
                }
                None => None,
            };
            match self.codec_type.as_str() {
                "video" => Ok(domain::Track::Video {
                    id: self.index,
                    codec: self
                        .codec_name
                        .clone()
                        .ok_or_else(|| Error::NoCodec(self.clone()))?,
                    duration,
                }),
                "audio" => Ok(domain::Track::Audio {
                    id: self.index,
                    codec: self
                        .codec_name
                        .clone()
                        .ok_or_else(|| Error::NoCodec(self.clone()))?,
                    duration,
                    language: self
                        .tags
                        .and_then(|tags| tags.language)
                        .and_then(|tag_string| {
                            domain::language::LanguageCode::try_from(tag_string.as_str()).ok()
                        }),
                }),
                "subtitle" => Ok(domain::Track::Subtitle {
                    id: self.index,
                    language: self.tags.clone().and_then(|tags| tags.language).and_then(
                        |tag_string| {
                            domain::language::LanguageCode::try_from(tag_string.as_str()).ok()
                        },
                    ),
                    // handler_name is the EXTERNAL_ID carrier. The default "SubtitleHandler"
                    // value set by ffmpeg is filtered out.
                    external_id: self
                        .tags
                        .and_then(|tags| tags.handler_name)
                        .filter(|h| h != "SubtitleHandler"),
                }),
                unknown => Err(Error::UnknownCodecType(unknown.to_string())),
            }
        }
    }

    #[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
    pub struct FfprobeStreamTags {
        pub language: Option<String>,
        pub handler_name: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use domain::Track;
    use std::{path::PathBuf, time::Duration};

    use domain::language::LanguageCode;

    use crate::get_tracks;

    #[tokio::test]
    async fn test_get_tracks() {
        let fixtures_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into();

        let h264_aac_nosub = fixtures_path.join("h264_aac_nosub.mp4");
        let h264_aac_1sub = fixtures_path.join("h264_aac_1sub.mp4");
        let h264_aac_3subs = fixtures_path.join("h264_aac_3subs.mp4");

        async {
            let mut result = get_tracks(&h264_aac_nosub)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by_key(|t| *t.id());
            assert_eq!(
                result,
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
        .await;

        async {
            let mut result = get_tracks(&h264_aac_1sub)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by_key(|t| *t.id());
            assert_eq!(
                result,
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
        .await;

        async {
            let mut result = get_tracks(&h264_aac_3subs)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by_key(|t| *t.id());
            assert_eq!(
                result,
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
        .await;
    }
}
