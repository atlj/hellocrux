pub struct TorrentInfo {
    added_on: usize,
    name: Box<str>,
    /// in bytes
    amount_left: usize,
    /// in bytes
    completed: usize,
    /// estimated completion date
    completion_on: usize,
    content_path: Box<str>,
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
    progress: usize,
    /// With torrent folder
    root_path: Box<str>,
    /// Without torrent folder
    save_path: Box<str>,
    /// in bytes
    size: usize,
    /// enum
    state: TorrentState,
    /// comma separated
    tags: Box<str>,
    /// In bytes
    uploaded: usize,
    upspeed: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
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
