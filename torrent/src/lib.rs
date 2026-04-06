mod api_types;
pub mod qbittorrent_client;
mod qbittorrent_web_api;

pub use api_types::into_domain::Error as ConversionError;

pub use qbittorrent_web_api::QBittorrentWebApiError;

pub use api_types::{TorrentContents, TorrentExtra, TorrentInfo, TorrentState};
