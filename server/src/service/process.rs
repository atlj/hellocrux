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
                    .send(())
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

#[derive(Debug)]
enum ProcessError {
    CantGetExtra,
    CantPrepare(
        // this is used when we log the error
        #[allow(dead_code)] crate::prepare::Error,
    ),
}

impl From<crate::prepare::Error> for ProcessError {
    fn from(value: crate::prepare::Error) -> Self {
        ProcessError::CantPrepare(value)
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self))
    }
}

impl std::error::Error for ProcessError {}

async fn process(media_dir: &Path, torrent: &TorrentInfo) -> Result<(), ProcessError> {
    let extra: TorrentExtra = torrent
        .as_ref()
        .try_into()
        .map_err(|_| ProcessError::CantGetExtra)?;

    match extra {
        TorrentExtra::Movie { ref metadata } => {
            crate::prepare::prepare_movie(media_dir, &torrent.save_path, metadata).await?;
        }
        TorrentExtra::Series {
            ref metadata,
            files_mapping_form,
        } => {
            crate::prepare::prepare_series(
                media_dir,
                &torrent.save_path,
                metadata,
                files_mapping_form.expect("files mapping form was None."),
            )
            .await?;
        }
    }

    Ok(())
}
