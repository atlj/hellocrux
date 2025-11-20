use std::path::PathBuf;

use log::{error, info};
use torrent::{TorrentExtra, qbittorrent_client::QBittorrentClientMessage};

pub type ProcessingListWatcher = crate::watch::Watcher<Box<[Box<str>]>>;

// 1. Get torrent list when it changes
//
// 2. Figure out which torrents needs to be processed
// based on is_done, its extra, and whether it is in processed list.
//
// 3. Add torrents that need to be processed to a list
//
// 4. Figure out which torrents are missing files
//
// 5. Update our processing list .
//
// 6. Process the torents that needs to be processed
//
// 7. Remove torrens that are done
//
// 8. send a signal to refresh the media library if we changed anything
/// A service that observes downloads and processes them
pub fn spawn(
    media_dir: PathBuf,
    crate::AppState {
        media_signal_watcher,
        mut download_signal_watcher,
        processing_list_watcher,
    }: crate::AppState,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // TODO REFACTOR THIS AS A WHOLE
        loop {
            let hashes: Box<[_]> = {
                let (torrents_to_process, torrents_with_missing_files) = {
                    // 1. Get torrent list when it changes
                    let torrents = download_signal_watcher.data.borrow_and_update().clone();
                    let processed_torrents = processing_list_watcher.data.borrow();

                    // 2. Figure out which torrents needs to be processed
                    // based on is_done, its extra, and whether it is in processed list.
                    let torrents_to_process: Box<_> = torrents
                        .clone()
                        .into_iter()
                        .filter(|torrent| torrent.state.is_done())
                        .filter(|torrent| {
                            match torrent.try_into().ok() as Option<TorrentExtra> {
                                Some(extra) => !extra.needs_file_mapping(),
                                None => {
                                    // TODO do proper error handling here
                                    error!("Couldn't extract extra from torrent category");
                                    false
                                }
                            }
                        })
                        .filter(|torrent| !processed_torrents.contains(&torrent.hash))
                        .collect();

                    // 3. Add torrents that need to be processed to a list
                    let updated_processed_torrents: Vec<Box<str>> = {
                        let mut vec = Vec::with_capacity(
                            processed_torrents.len() + torrents_to_process.len(),
                        );
                        vec.extend_from_slice(&processed_torrents);
                        vec.extend(
                            torrents_to_process
                                .iter()
                                .map(|torrent| torrent.hash.clone()),
                        );

                        vec
                    };

                    // 4. Figure out which torrents are missing files
                    // TODO, instead of checking for missing files here, check them by the end
                    let torrents_with_missing_files: Box<[Box<str>]> = torrents
                        .into_iter()
                        .filter_map(|torrent| {
                            if matches!(torrent.state, torrent::TorrentState::MissingFiles) {
                                Some(torrent.hash.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    drop(processed_torrents);
                    // TODO: drop let _
                    // 5. Update our processing list .
                    let _ = processing_list_watcher
                        .updater
                        .send(updated_processed_torrents.into());

                    (torrents_to_process, torrents_with_missing_files)
                };

                // 6. Process the torents that needs to be processed
                let futures =
                    torrents_to_process
                        .into_iter()
                        .map(async |torrent| -> Option<Box<str>> {
                            let extra: TorrentExtra = torrent
                                .as_ref()
                                .try_into()
                                .inspect_err(|err| {
                                    error!("Couldn't extract extra from torrent's category. {err}")
                                })
                                .ok()?;
                            let title = extra.metadata_ref().title.clone();

                            info!("Preparing torrent named {}.", &torrent.name);

                            match extra {
                                TorrentExtra::Movie { ref metadata } => {
                                    crate::prepare::prepare_movie(
                                        &media_dir,
                                        &torrent.content_path,
                                        metadata,
                                    )
                                    .await
                                    .inspect_err(|err| {
                                        error!(
                                            "Couldn't prepare movie with title {}. Reason: {err}.",
                                            extra.metadata_ref().title
                                        )
                                    })
                                    .ok()?;
                                }
                                TorrentExtra::Series {
                                    ref metadata,
                                    files_mapping_form,
                                } => {
                                    crate::prepare::prepare_series(
                                        &media_dir,
                                        &torrent.save_path,
                                        metadata,
                                        files_mapping_form.expect("files mapping form was None."),
                                    )
                                    .await
                                    .inspect_err(|err| {
                                        // TODO delete the files if this happens
                                        error!(
                                            "Couldn't prepare series with title {}. Reason: {err}.",
                                            title
                                        )
                                    })
                                    .ok()?;
                                }
                            }

                            Some(torrent.hash.clone())
                        });

                futures::future::join_all(futures)
                    .await
                    .into_iter()
                    .flatten()
                    .chain(torrents_with_missing_files)
                    .collect()
            };

            // 7. Remove torrens that are done
            let removal_futures = hashes.iter().map(async |hash| {
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

            let did_media_library_change = !hashes.is_empty();

            // TODO do we need this?
            if download_signal_watcher.data.changed().await.is_err() {
                break;
            }

            // 8. send a signal to refresh the media library
            if did_media_library_change {
                let _ = media_signal_watcher
                    .signal_sender
                    .send(())
                    .await
                    .inspect_err(|_| error!("Media library watcher loop was dropped."));
            }
        }

        unreachable!("Torrent channel was dropped")
    })
}
