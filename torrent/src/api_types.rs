use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TorrentInfo {
    added_on: usize,
    name: Box<str>,
    /// in bytes
    amount_left: usize,
    /// in bytes
    completed: usize,
    /// estimated completion date
    completion_on: isize,
    content_path: PathBuf,
    /// in bytes
    dlspeed: usize,
    /// in bytes
    downloaded: usize,
    /// in seconds
    eta: usize,
    hash: Box<str>,
    magnet_uri: Box<str>,
    num_seeds: usize,
    /// percentage/100
    progress: f32,
    /// With torrent folder
    root_path: PathBuf,
    /// Without torrent folder
    save_path: PathBuf,
    /// in bytes
    size: usize,
    /// enum
    state: TorrentState,
    /// comma separated
    #[serde(with = "comma_separated_list_parser")]
    tags: Box<[Box<str>]>,
    /// In bytes
    uploaded: usize,
    upspeed: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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
