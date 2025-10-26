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
    state: Box<str>,
    /// comma separated
    tags: Box<str>,
    /// In bytes
    uploaded: usize,
    upspeed: usize,
}
