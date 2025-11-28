use std::{collections::HashMap, path::PathBuf};

use domain::Media;
use log::{error, info};

pub enum MediaSignal {
    CrawlAll,
    CrawlCertainMedia { media_id: String },
}

pub type MediaSignalWatcher = crate::signal::SignalWatcher<MediaSignal, HashMap<String, Media>>;
pub type MediaSignalReceiver = crate::signal::SignalReceiver<MediaSignal, HashMap<String, Media>>;

/// A service that crawls the media library
pub async fn spawn(
    media_dir: PathBuf,
    mut media_signal_receiver: MediaSignalReceiver,
    media_signal_watcher: MediaSignalWatcher,
) -> tokio::task::JoinHandle<()> {
    let handle = tokio::spawn(async move {
        while let Some(signal) = media_signal_receiver.signal_receiver.recv().await {
            tokio::fs::create_dir_all(&media_dir)
                .await
                .expect("Couldn't create media dir");

            let entries = match signal {
                MediaSignal::CrawlAll => {
                    info!("Crawling media items");

                    let entries: HashMap<String, Media> =
                        crate::crawl::crawl_all_folders(media_dir.to_string_lossy().as_ref())
                            .await
                            .unwrap_or(HashMap::new());

                    info!("Found {:#?} media items", entries.len());
                    entries
                }
                MediaSignal::CrawlCertainMedia { media_id } => todo!(),
            };

            if media_signal_receiver.updater.send(entries).is_err() {
                error!("Media list receiver was dropped. Can't update the media library")
            }
        }
    });

    media_signal_watcher
        .signal_sender
        .send(MediaSignal::CrawlAll)
        .await
        .expect("Update request listener was dropped. Is media watcher loop alive?");

    handle
}
