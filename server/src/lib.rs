pub mod download_handlers;
pub mod prepare;
pub mod service;
pub mod watch;
use std::path::PathBuf;

use clap::Parser;

use service::media::MediaSignalWatcher;
use service::torrent::DownloadSignalWatcher;

#[derive(Parser, Clone)]
#[command()]
pub struct Args {
    /// Path to the media dir
    #[arg(short, long, default_value = "./media")]
    pub media_dir: PathBuf,
}

pub type ProcessingListWatcher = watch::Watcher<Box<[Box<str>]>>;

#[derive(Clone)]
pub struct AppState {
    pub media_signal_watcher: MediaSignalWatcher,
    pub download_signal_watcher: DownloadSignalWatcher,
    pub processing_list_watcher: ProcessingListWatcher,
}

pub type State = axum::extract::State<AppState>;
