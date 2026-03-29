use std::path::Path;

use crate::spawn::ffprobe;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Track {
    Video {
        id: usize,
        codec: String,
        duration: Option<std::time::Duration>,
    },
    Audio {
        id: usize,
        codec: String,
        duration: Option<std::time::Duration>,
        language: Option<domain::language::LanguageCode>,
    },
    Subtitle {
        id: usize,
        language: Option<domain::language::LanguageCode>,
    },
}

impl Track {
    pub fn id(&self) -> &usize {
        match self {
            Track::Video { id, .. } => id,
            Track::Audio { id, .. } => id,
            Track::Subtitle { id, .. } => id,
        }
    }
}

pub async fn get_tracks(
    media_file: impl AsRef<Path>,
) -> crate::Result<impl Iterator<Item = crate::Result<Track>>> {
    let result_string = ffprobe([
        // Error on empty
        "-v",
        "error",
        // Output JSON
        "-print_format",
        "json",
        // Only show relevant entries
        "-show_entries",
        "stream=index,codec_name,codec_type,duration:stream_tags=language,EXTERNAL_ID",
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

    impl TryInto<crate::Track> for FfprobeStream {
        type Error = Error;

        fn try_into(self) -> Result<crate::Track, Self::Error> {
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
                "video" => Ok(crate::Track::Video {
                    id: self.index,
                    codec: self
                        .codec_name
                        .clone()
                        .ok_or_else(|| Error::NoCodec(self.clone()))?,
                    duration,
                }),
                "audio" => Ok(crate::Track::Audio {
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
                "subtitle" => Ok(crate::Track::Subtitle {
                    id: self.index,
                    language: self.tags.clone().and_then(|tags| tags.language).and_then(
                        |tag_string| {
                            domain::language::LanguageCode::try_from(tag_string.as_str()).ok()
                        },
                    ),
                }),
                unknown => Err(Error::UnknownCodecType(unknown.to_string())),
            }
        }
    }

    #[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
    pub struct FfprobeStreamTags {
        pub language: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use domain::language::LanguageCode;

    use crate::{Track, get_tracks};

    #[tokio::test]
    async fn test_get_tracks() {
        let fixtures_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into();

        let h264_aac_nosub = fixtures_path.join("h264_aac_nosub.mkv");
        let h265_flac_3subs = fixtures_path.join("h265_flac_3subs.mkv");
        let hevc_aac_1sub = fixtures_path.join("hevc_aac_1sub.mkv");

        async {
            let mut result = get_tracks(&h264_aac_nosub)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by(|a, b| a.id().cmp(&b.id()));
            assert_eq!(
                result,
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
                    }
                ]
            );
        }
        .await;

        async {
            let mut result = get_tracks(&h265_flac_3subs)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by(|a, b| a.id().cmp(&b.id()));
            assert_eq!(
                result,
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
                    },
                    Track::Subtitle {
                        id: 3,
                        language: Some(LanguageCode::French),
                    },
                    Track::Subtitle {
                        id: 4,
                        language: None,
                    }
                ]
            );
        }
        .await;

        async {
            let mut result = get_tracks(&hevc_aac_1sub)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            result.sort_by(|a, b| a.id().cmp(&b.id()));
            assert_eq!(
                result,
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
                    }
                ]
            );
        }
        .await;
    }
}
