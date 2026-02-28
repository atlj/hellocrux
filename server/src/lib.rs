pub mod crawl;
pub mod dir;
pub mod download_handlers;
pub mod ffmpeg;
pub mod prepare;
pub mod service;
pub mod signal;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;

#[derive(Parser, Clone)]
#[command(about = "Launches a streamy server.")]
pub struct Args {
    /// Path to the media library.
    #[arg(short, long, default_value = "./media")]
    pub media_dir: PathBuf,

    /// The name displayed when server is automatically discovered by a client.
    /// Defaults to your machine's host name.
    #[arg(long, default_value_t = Args::default_name())]
    pub name: String,
}

impl Args {
    fn default_name() -> String {
        gethostname::gethostname().to_string_lossy().to_string()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub media_dir: Arc<Path>,
    pub media_signal_watcher: service::media::MediaSignalWatcher,
    pub download_signal_watcher: service::download::DownloadSignalWatcher,
    pub processing_list_watcher: service::process::ProcessingListWatcher,
}

pub type State = axum::extract::State<AppState>;
