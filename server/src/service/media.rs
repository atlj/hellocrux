use std::path::PathBuf;

use domain::Media;
use log::{error, info};

pub type MediaSignalWatcher = crate::signal::SignalWatcher<(), Box<[Media]>>;
pub type MediaSignalReceiver = crate::signal::SignalReceiver<(), Box<[Media]>>;

/// A service that crawls the media library
pub async fn spawn(
    media_dir: PathBuf,
    mut media_signal_receiver: MediaSignalReceiver,
    media_signal_watcher: MediaSignalWatcher,
) -> tokio::task::JoinHandle<()> {
    let handle = tokio::spawn(async move {
        while media_signal_receiver.signal_receiver.recv().await.is_some() {
            tokio::fs::create_dir_all(&media_dir)
                .await
                .expect("Couldn't create media dir");

            info!("Crawling media items");

            let entries: Box<[Media]> =
                crate::crawl::crawl_all_folders(media_dir.to_string_lossy().as_ref())
                    .await
                    .unwrap_or(Box::new([]));

            info!("Found {:#?} media items", entries.len());

            if media_signal_receiver.updater.send(entries).is_err() {
                error!("Media list receiver was dropped. Can't update the media library")
            }
        }
    });

    media_signal_watcher
        .signal_sender
        .send(())
        .await
        .expect("Update request listener was dropped. Is media watcher loop alive?");

    handle
}
