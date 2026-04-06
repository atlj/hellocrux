use std::path::{Path, PathBuf};

use log::{error, info};
use torrent::{TorrentExtra, TorrentInfo, qbittorrent_client::QBittorrentClientMessage};

pub type ProcessingListWatcher = crate::signal::Watcher<Box<[Box<str>]>>;

/// A service that observes downloads and processes them
pub fn spawn(
    media_dir: PathBuf,
    crate::AppState {
        media_signal_watcher,
        mut download_signal_watcher,
        processing_list_watcher,
        ..
    }: crate::AppState,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let ids_to_remove: Box<[_]> = {
                // 1. Get torrent list when it changes
                let torrents = download_signal_watcher.data.borrow_and_update().clone();
                let processed_torrents = processing_list_watcher.data.borrow().clone();

                let faulty_ids_to_remove: Box<[Box<str>]> = torrents
                    .iter()
                    .filter_map(|torrent| {
                        if torrent.state.is_faulty() {
                            return Some(torrent.hash.clone());
                        }

                        None
                    })
                    .collect();

                // 2. Figure out which torrents needs to be processed
                let torrents_to_process: Box<_> = torrents
                        .into_iter()
                        .filter(|torrent| {
                            match torrent.should_process() {
                                Ok(should_process) => should_process,
                                Err(err) => {
                                    error!("Couldn't figure out if torrent needs to be processed. Reason: {err}");
                                    false
                                }
                            }
                        })
                        .filter(|torrent| !processed_torrents.contains(&torrent.hash))
                        .collect();

                // 3. Add torrents that need to be processed to a list
                let updated_processed_torrents: Vec<Box<str>> = {
                    let mut vec =
                        Vec::with_capacity(processed_torrents.len() + torrents_to_process.len());
                    vec.extend_from_slice(&processed_torrents);
                    vec.extend(
                        torrents_to_process
                            .iter()
                            .map(|torrent| torrent.hash.clone()),
                    );

                    vec
                };

                // 4. Update our processing list.
                processing_list_watcher
                    .updater
                    .send(updated_processed_torrents.into())
                    .expect(
                        "Couldn't update internal torrent processing list. Was channel dropped?",
                    );

                // 5. Process the torents that needs to be processed
                let process_futures = torrents_to_process.into_iter().map(
                    async |torrent| -> Result<Box<str>, ProcessError> {
                        info!("Preparing torrent named {}", &torrent.name);

                        process(&media_dir, &torrent)
                            .await
                            .inspect_err(|err| {
                                error!(
                                    "Couldn't prepare torrent with id {}. Reason: {err}",
                                    torrent.hash
                                )
                            })
                            .inspect(|_| {
                                info!("Done preparing torrent named {}", &torrent.name);
                            })
                            .map(|_| torrent.hash)
                    },
                );

                futures::future::join_all(process_futures)
                    .await
                    .into_iter()
                    .flatten()
                    .chain(faulty_ids_to_remove)
                    .collect()
            };

            // 6. Remove torrens that are done
            let removal_futures = ids_to_remove.iter().map(async |hash| {
                let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

                download_signal_watcher
                    .signal_sender
                    .send(QBittorrentClientMessage::RemoveTorrent {
                        id: hash.clone(),
                        result_sender,
                    })
                    .await
                    .inspect_err(|err| error!("QBittorrent Client was dropped. Reason: {err}"))
                    .ok()?;

                result_receiver
                    .await
                    .inspect_err(|err| error!("QBittorrent Client was dropped. Reason: {err}"))
                    .ok()?
                    .inspect_err(|err| {
                        error!("Couldn't remove torrent with hash {hash}. Reason: {err}")
                    })
                    .ok()?;

                Some(())
            });

            futures::future::join_all(removal_futures).await;

            let did_media_library_change = !ids_to_remove.is_empty();

            // 7. send a signal to refresh the media library
            if did_media_library_change {
                let _ = media_signal_watcher
                    .signal_sender
                    // TODO: only send crawl message for new data
                    .send(super::media::MediaSignal::CrawlAll)
                    .await
                    .inspect_err(|_| error!("Media library watcher loop was dropped."));
            }

            download_signal_watcher
                .data
                .changed()
                .await
                .expect("Download channel was closed");
        }
    })
}

#[derive(thiserror::Error, Debug)]
enum ProcessError {
    #[error("Can't get extra from torrent")]
    CantGetExtra {
        torrent_info: Box<TorrentInfo>,
        inner: torrent::ConversionError,
    },
    #[error("Can't move torrent. {0}")]
    CantMove(crate::moving::Error),
}

impl From<crate::moving::Error> for ProcessError {
    fn from(value: crate::moving::Error) -> Self {
        ProcessError::CantMove(value)
    }
}

async fn process(media_dir: &Path, torrent: &TorrentInfo) -> Result<(), ProcessError> {
    let extra: TorrentExtra =
        torrent
            .as_ref()
            .try_into()
            .map_err(|err| ProcessError::CantGetExtra {
                torrent_info: Box::new(torrent.clone()),
                inner: err,
            })?;

    match extra {
        TorrentExtra::Movie { ref metadata } => {
            crate::moving::generate_movie_media(media_dir, &torrent.save_path, metadata).await?;
        }
        TorrentExtra::Series {
            ref metadata,
            files_mapping_form,
        } => {
            crate::moving::generate_series_media(
                media_dir,
                &torrent.save_path,
                files_mapping_form.expect("files mapping form was None."),
                metadata,
            )
            .await?;
        }
    }

    Ok(())
}
