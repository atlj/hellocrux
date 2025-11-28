use std::{collections::HashMap, path::PathBuf};

use domain::Media;
use log::{error, info, warn};

pub enum MediaSignal {
    CrawlAll,
    CrawlPartial { media_id: String },
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
                MediaSignal::CrawlPartial { media_id } => {
                    let mut entries = media_signal_watcher.data.borrow().clone();

                    info!("Crawling media item with id {media_id}");

                    match crate::crawl::crawl_folder(
                        media_dir.join(&media_id).to_string_lossy().as_ref(),
                    )
                    .await
                    {
                        Some(new_media) => {
                            info!("Updated media item with id {media_id}");
                            entries.insert(media_id, new_media);
                        }
                        None => {
                            entries.remove(&media_id);
                            warn!("Recrawled entry with id {media_id} but it was gone.");
                        }
                    };

                    entries
                }
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
