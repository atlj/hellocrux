pub mod download_handlers;
pub mod media;
pub mod prepare;
use std::path::PathBuf;

use clap::Parser;
use domain::Media;
use torrent::{TorrentInfo, qbittorrent_client::QBittorrentClientMessage};

#[derive(Parser, Clone)]
#[command()]
pub struct Args {
    /// Path to the media dir
    #[arg(short, long, default_value = "./media")]
    pub media_dir: PathBuf,
}

#[derive(Clone)]
pub struct AppState {
    pub media_channels: (
        tokio::sync::mpsc::Sender<()>,
        tokio::sync::watch::Receiver<Box<[Media]>>,
    ),
    pub download_channels: (
        tokio::sync::mpsc::Sender<QBittorrentClientMessage>,
        tokio::sync::watch::Receiver<Box<[TorrentInfo]>>,
    ),
}

pub type State = axum::extract::State<AppState>;
