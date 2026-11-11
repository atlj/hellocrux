use std::collections::VecDeque;

use log::{error, info, warn};

pub enum PrepareMessage {
    Prepare((domain::MediaMetaData, domain::MediaIdentifier)),
    SelectTracks(domain::TrackSelectionItem),
    Done((domain::MediaMetaData, domain::MediaIdentifier)),
}

pub type PreparingListWatcher = crate::signal::SignalWatcher<
    PrepareMessage,
    (
        Option<domain::PreparingProgress>,
        Vec<(domain::MediaMetaData, domain::MediaIdentifier)>,
        Vec<(domain::MediaMetaData, domain::TrackSelectionItem)>,
    ),
>;
pub type PreparingListReceiver = crate::signal::SignalReceiver<
    PrepareMessage,
    (
        Option<domain::PreparingProgress>,
        Vec<(domain::MediaMetaData, domain::MediaIdentifier)>,
        Vec<(domain::MediaMetaData, domain::TrackSelectionItem)>,
    ),
>;

/// Makes media files compatible
pub fn spawn(
    mut signal_receiver: PreparingListReceiver,
    crate::AppState {
        media_dir,
        media_signal_watcher,
        preparing_list_watcher,
        ..
    }: crate::AppState,
) -> tokio::task::JoinHandle<()> {
    let mut preparing_queue: VecDeque<(
        domain::MediaMetaData,
        domain::MediaIdentifier,
        Vec<ffmpeg::TrackSelection>,
    )> = VecDeque::with_capacity(50);
    let mut track_selection_wait_queue: VecDeque<(
        domain::MediaMetaData,
        domain::TrackSelectionItem,
    )> = VecDeque::with_capacity(50);

    let mut task: Option<tokio::task::JoinHandle<()>> = None;

    tokio::spawn(async move {
        while let Some(signal) = signal_receiver.signal_receiver.recv().await {
            // 1. Handle Message
            match signal {
                PrepareMessage::Prepare((media_metadata, media_identifier)) => {
                    // 1. See if we can queue the item immediately
                    let default_track_selections = match crate::prepare::default_track_selections(
                        &media_identifier.path().media,
                    )
                    .await
                    {
                        Ok(selection) => selection,
                        Err(err) => {
                            error!(
                                "Couldn't get tracks of media {media_identifier:#?}. Ignoring it. {err}"
                            );
                            continue;
                        }
                    };

                    // TODO: dedupe
                    match default_track_selections {
                        Some(track_selections) => {
                            // TODO instead of doing a search here, find a more clever way
                            // 2a. Media is good to go, enqueue it if it's not here already
                            if preparing_queue
                                .iter()
                                .find(|item| item.1 == media_identifier)
                                .is_none()
                            {
                                preparing_queue.push_back((
                                    media_metadata,
                                    media_identifier,
                                    track_selections,
                                ));
                            }
                        }
                        None => {
                            // 2b. Media needs user input, put it to pending list.
                            let tracks = match ffmpeg::get_tracks(&media_identifier.path().media)
                                .await
                                .and_then(|tracks| tracks.collect::<Result<Vec<_>, _>>())
                            {
                                Ok(tracks) => tracks,
                                Err(err) => {
                                    error!(
                                        "Couldn't get tracks of media {media_identifier:#?}. Ignoring it. {err}"
                                    );
                                    continue;
                                }
                            };

                            // TODO instead of doing a search here, find a more clever way
                            if track_selection_wait_queue
                                .iter()
                                .find(|item| item.1.media == media_identifier)
                                .is_none()
                            {
                                info!("Awaiting track selection to prepare {media_identifier:#?}.");
                                track_selection_wait_queue.push_back((
                                    media_metadata,
                                    domain::TrackSelectionItem {
                                        media: media_identifier,
                                        tracks,
                                    },
                                ));
                            }
                        }
                    }
                }
                PrepareMessage::SelectTracks(domain::TrackSelectionItem {
                    media: user_provided_media,
                    tracks,
                }) => {
                    // 1. Get the right item from our internal list
                    let Some((metadata, item)) = track_selection_wait_queue
                        .iter()
                        .enumerate()
                        .find(|(_, current)| {
                            current.1.media.id() == user_provided_media.id()
                                && current.1.media.episode_identifier()
                                    == user_provided_media.episode_identifier()
                        })
                        .map(|(index, _)| index)
                        .and_then(|index| track_selection_wait_queue.remove(index))
                    else {
                        warn!(
                            "Provided track selection for {user_provided_media:#?} but looks like tracks were already provided. Ignoring it."
                        );
                        continue;
                    };

                    let media = &item.media;

                    // 2. Convert user provided tracks to selections
                    let selections = tracks
                        .into_iter()
                        .map(|track| {
                            ffmpeg::TrackExt::into_selection(
                                track,
                                media.path().media.clone().into(),
                            )
                        })
                        .collect::<Vec<_>>();

                    // 3. If all good, put the item to queue
                    // TODO: dedupe
                    preparing_queue.push_back((metadata, item.media, selections));
                }
                PrepareMessage::Done((prepared_items_metadata, media_identifier)) => {
                    // 1. Last task was done, ready to run next item
                    task = None;

                    // 2. Remove from queue
                    match preparing_queue
                        .front()
                        .map(|(_, first_id, _)| first_id == &media_identifier)
                    {
                        Some(true) => {
                            preparing_queue.pop_front();
                        }
                        _ => {
                            error!(
                                "Processing {media_identifier:#?} was done but it was already removed from the preparing list. Check server code."
                            );
                        }
                    }

                    // 3. Tell media lib to recrawl (this may lead to an infinite loop)
                    let queue_has_another_item_with_same_metadata = preparing_queue
                        .iter()
                        .map(|(metadata, _, _)| metadata)
                        .chain(
                            track_selection_wait_queue
                                .iter()
                                .map(|(metadata, _)| metadata),
                        )
                        .find(|metadata| **metadata == prepared_items_metadata)
                        .is_some();

                    // TODO implement loop detection
                    if !queue_has_another_item_with_same_metadata {
                        if let Err(err) = media_signal_watcher
                            .signal_sender
                            .send(crate::service::media::MediaSignal::CrawlPartial {
                                media_id: media_identifier.id().to_string(),
                            })
                            .await
                        {
                            error!(
                                "Prepared media item {media_identifier:#?} but couldn't tell media service to recrawl it due to {err}. Restart the server."
                            );
                        }
                    } else {
                        info!(
                            "Prepared media item {media_identifier:#?} but didn't recrawl it yet since there are more items from same media waiting to be prepared."
                        );
                    }
                }
            }

            // 2. Announce we've updated the tasks
            signal_receiver
                .updater
                .send_modify(|(_, preparing, track_selection)| {
                    *preparing = preparing_queue
                        .iter()
                        .cloned()
                        .map(|(metadata, id, _)| (metadata, id))
                        .flat_map(|(metadata, id)| {
                            id.strip_prefix(&media_dir).map(|id| (metadata, id))
                        })
                        .collect();

                    *track_selection = track_selection_wait_queue
                        .iter()
                        .cloned()
                        .flat_map(|(metadata, item)| {
                            item.strip_prefix(&media_dir).map(|id| (metadata, id))
                        })
                        .collect();
                });

            // 3. Start working on a task if none present
            if task.is_none()
                && let Some((head_metadata, head_id, head_track_selections)) =
                    preparing_queue.front().cloned()
            {
                let sender = preparing_list_watcher.signal_sender.clone();
                let media_dir = media_dir.clone();
                let preparing_updater = signal_receiver.updater.clone();
                task = Some(tokio::spawn(async move {
                    info!("Preparing {head_id:#?}");

                    let (progress_sender, mut progress_watcher) = tokio::sync::watch::channel(None);

                    let progress_watch_handle = tokio::spawn(async move {
                        loop {
                            let progress = progress_watcher.borrow().clone();
                            preparing_updater.send_modify(|(progress_state, _, _)| {
                                *progress_state = progress;
                            });

                            if progress_watcher.changed().await.is_err() {
                                return;
                            }
                        }
                    });

                    if let Err(err) = crate::prepare::prepare_media(
                        &head_id,
                        head_track_selections,
                        media_dir,
                        progress_sender,
                    )
                    .await
                    {
                        error!("Couldn't prepare media with id {head_id:#?}. {err}");
                    }

                    progress_watch_handle.abort();

                    if let Err(err) = sender
                        .send(PrepareMessage::Done((head_metadata, head_id.clone())))
                        .await
                    {
                        error!(
                            "Prepared {head_id:#?} but couldn't tell prepare service about it. {err}"
                        );
                    }
                }))
            }
        }
    })
}
