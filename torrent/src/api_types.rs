use std::path::PathBuf;

use domain::{MediaMetaData, series::SeriesFileMapping};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TorrentExtra {
    Movie {
        metadata: MediaMetaData,
    },
    Series {
        metadata: MediaMetaData,
        files_mapping: Option<SeriesFileMapping>,
    },
}

impl TorrentExtra {
    pub fn new(metadata: MediaMetaData, is_series: bool) -> Self {
        match is_series {
            true => Self::Series {
                metadata,
                files_mapping: None,
            },
            false => Self::Movie { metadata },
        }
    }
    pub fn needs_file_mapping(&self) -> bool {
        match self {
            TorrentExtra::Movie { .. } => false,
            TorrentExtra::Series { files_mapping, .. } => files_mapping.is_none(),
        }
    }
    pub fn metadata(self) -> MediaMetaData {
        match self {
            TorrentExtra::Movie { metadata } => metadata,
            TorrentExtra::Series { metadata, .. } => metadata,
        }
    }

    pub fn metadata_ref(&self) -> &MediaMetaData {
        match self {
            TorrentExtra::Movie { metadata } => metadata,
            TorrentExtra::Series { metadata, .. } => metadata,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TorrentInfo {
    pub added_on: usize,
    pub name: Box<str>,
    /// in bytes
    pub amount_left: usize,
    /// Category of the torrent.
    /// We use this internally to store `TorrentExtra` as a b64 encoded JSON string.
    pub category: Box<str>,
    /// in bytes
    pub completed: usize,
    /// estimated completion date
    pub completion_on: isize,
    pub content_path: PathBuf,
    /// in bytes
    pub dlspeed: usize,
    /// in bytes
    pub downloaded: usize,
    /// in seconds
    pub eta: usize,
    pub hash: Box<str>,
    pub magnet_uri: Box<str>,
    pub num_seeds: usize,
    /// percentage/100
    pub progress: f32,
    /// With torrent folder
    pub root_path: PathBuf,
    /// Without torrent folder
    pub save_path: PathBuf,
    /// in bytes
    pub size: usize,
    /// enum
    pub state: TorrentState,
    /// comma separated
    #[serde(with = "comma_separated_list_parser")]
    pub tags: Box<[Box<str>]>,
    /// In bytes
    pub uploaded: usize,
    pub upspeed: usize,
}

impl AsRef<TorrentInfo> for TorrentInfo {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TorrentState {
    /// Some error occurred, applies to paused torrents
    #[serde(rename = "error")]
    Error,
    /// Torrent data files is missing
    #[serde(rename = "missingFiles")]
    MissingFiles,
    /// Torrent is being seeded and data is being transferred
    #[serde(rename = "uploading")]
    Uploading,
    /// Torrent is paused and has finished downloading
    #[serde(rename = "pausedUP")]
    PausedUP,
    /// Queuing is enabled and torrent is queued for upload
    #[serde(rename = "queuedUP")]
    QueuedUP,
    /// Torrent is being seeded, but no connection were made
    #[serde(rename = "stalledUP")]
    StalledUP,
    /// Torrent has finished downloading and is being checked
    #[serde(rename = "checkingUP")]
    CheckingUP,
    /// Torrent is forced to uploading and ignore queue limit
    #[serde(rename = "forcedUP")]
    ForcedUP,
    /// Torrent is allocating disk space for download
    #[serde(rename = "allocating")]
    Allocating,
    /// Torrent is being downloaded and data is being transferred
    #[serde(rename = "downloading")]
    Downloading,
    /// Torrent has just started downloading and is fetching metadata
    #[serde(rename = "metaDL")]
    MetaDL,
    /// Torrent is paused and has NOT finished downloading
    #[serde(rename = "pausedDL")]
    PausedDL,
    /// Queuing is enabled and torrent is queued for download
    #[serde(rename = "queuedDL")]
    QueuedDL,
    /// Torrent is being downloaded, but no connection were made
    #[serde(rename = "stalledDL")]
    StalledDL,
    /// Same as checkingUP, but torrent has NOT finished downloading
    #[serde(rename = "checkingDL")]
    CheckingDL,
    /// Torrent is forced to downloading to ignore queue limit
    #[serde(rename = "forcedDL")]
    ForcedDL,
    /// Checking resume data on qBt startup
    #[serde(rename = "checkingResumeData")]
    CheckingResumeData,
    /// Torrent is moving to another location
    #[serde(rename = "moving")]
    Moving,
    /// Unknown status
    #[serde(rename = "unknown")]
    Unknown,
    // Not included in docs
    #[serde(rename = "stoppedDL")]
    StoppedDL,
}

impl TorrentState {
    pub fn should_stop(&self) -> bool {
        matches!(
            self,
            Self::Error
                | Self::Uploading
                | Self::MissingFiles
                | Self::StoppedDL
                | Self::PausedUP
                | Self::PausedDL
                | Self::StalledUP
        )
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Uploading | Self::StalledUP)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::PausedDL | Self::PausedUP | Self::StoppedDL)
    }
}

mod comma_separated_list_parser {
    use serde::{Deserializer, Serializer, de::Visitor};

    pub fn serialize<S>(input: &[Box<str>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&input.join(","))
    }

    struct CommaSeparatedListVisitor;

    impl<'de> Visitor<'de> for CommaSeparatedListVisitor {
        type Value = Box<[Box<str>]>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A comma separated list")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.split(',').map(|elem| elem.into()).collect())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Box<[Box<str>]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CommaSeparatedListVisitor)
    }

    #[cfg(test)]
    mod tests {
        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        struct CustomData {
            #[serde(with = "super")]
            comma_separated_list: Box<[Box<str>]>,
        }

        #[test]
        fn test_comma_separated_list_deserialize() {
            let str = r#"{"comma_separated_list": "val1,hey,val2"}"#;
            assert_eq!(
                serde_json::from_str::<CustomData>(str).unwrap(),
                CustomData {
                    comma_separated_list: Box::new(["val1".into(), "hey".into(), "val2".into()])
                }
            )
        }

        #[test]
        fn test_comma_separated_list_serialize() {
            assert_eq!(
                serde_json::to_string(&CustomData {
                    comma_separated_list: Box::new(["hey".into(), "there".into()]),
                })
                .unwrap(),
                r#"{"comma_separated_list":"hey,there"}"#
            )
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TorrentContents {
    pub index: usize,
    pub is_seed: Option<bool>,
    pub name: Box<str>,
    pub piece_range: Box<[isize]>,
    pub priority: isize,
    pub progress: f32,
    pub size: usize,
    pub availability: f32,
}

pub mod into_domain {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE};
    use std::fmt::Display;

    use crate::TorrentInfo;
    use domain::Download;

    use super::TorrentExtra;

    #[derive(Debug)]
    pub enum Error {
        CantDecodeUsingBase64(Box<str>),
        CantConvertBase64BytesToString(Box<str>),
        CantDeserializeStringToTorrentExtra(Box<str>),
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Error::CantDecodeUsingBase64(msg) => msg,
                Error::CantConvertBase64BytesToString(msg) => msg,
                Error::CantDeserializeStringToTorrentExtra(msg) => msg,
            })
        }
    }

    impl TryFrom<&TorrentInfo> for TorrentExtra {
        type Error = Error;

        fn try_from(value: &TorrentInfo) -> Result<Self, Self::Error> {
            let extra_str_bytes = URL_SAFE.decode(value.category.as_bytes()).map_err(|err| {
                Error::CantDecodeUsingBase64(
                format!(
                    "Couldn't decode torrent named {}'s category ({}) using base64. Reason: {err}",
                    value.name, value.category
                ).into()
                )
            })?;

            let extra_string = str::from_utf8(&extra_str_bytes).map_err(|err| {
                Error::CantConvertBase64BytesToString(

                        format!(
                            "Couldn't convert b64 decoded bytes from torrent with name {}'s category ({}) to a string. Reason: {err}",
                            value.name,
                            value.category
                        ).into()
                )
                    })?;

            serde_json::from_str(extra_string).map_err(|err| {
                Error::CantDeserializeStringToTorrentExtra(
                    format!(
                        "Couldn't deserialize torrent named {}'s category ({}). Reason: {err}",
                        value.name, value.category
                    )
                    .into(),
                )
            })
        }
    }

    impl From<TorrentInfo> for Download {
        fn from(val: TorrentInfo) -> Self {
            let extra: Option<TorrentExtra> = val.as_ref().try_into().ok();

            let title = extra
                .as_ref()
                .map(|download_form| download_form.metadata_ref().title.clone().into_boxed_str())
                .unwrap_or(val.name);

            let needs_file_mapping = extra
                .as_ref()
                .map(TorrentExtra::needs_file_mapping)
                .unwrap_or(false);

            Download {
                id: val.hash,
                title,
                progress: val.progress,
                is_paused: val.state.is_paused(),
                needs_file_mapping,
            }
        }
    }
}
