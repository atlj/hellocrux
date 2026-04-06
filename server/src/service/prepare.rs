use std::collections::VecDeque;

use log::{error, info};

pub enum PrepareMessage {
    Prepare(domain::MediaIdentifier),
    SelectTracks(domain::TrackSelectionItem),
    Done(domain::MediaIdentifier),
}

pub type PreparingListWatcher = crate::signal::SignalWatcher<
    PrepareMessage,
    (
        Vec<domain::MediaIdentifier>,
        Vec<domain::TrackSelectionItem>,
    ),
>;
pub type PreparingListReceiver = crate::signal::SignalReceiver<
    PrepareMessage,
    (
        Vec<domain::MediaIdentifier>,
        Vec<domain::TrackSelectionItem>,
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
    let mut preparing_queue: VecDeque<(domain::MediaIdentifier, Vec<ffmpeg::TrackSelection>)> =
        VecDeque::with_capacity(50);
    let mut track_selection_wait_queue = VecDeque::with_capacity(50);

    let mut task: Option<tokio::task::JoinHandle<()>> = None;

    tokio::spawn(async move {
        while let Some(signal) = signal_receiver.signal_receiver.recv().await {
            // 1. Handle Message
            match signal {
                PrepareMessage::Prepare(media_identifier) => {
                    // 1. See if we can queue the item immediately
                    let default_track_selections = match crate::prepare::default_track_selections(
                        &media_identifier.path().media,
                    )
                    .await
                    {
                        Ok(selection) => selection,
                        Err(err) => {
                            error!(
                                "Couldn't determine default track selection for media {media_identifier:#?}. Ignoring it. {err}"
                            );
                            continue;
                        }
                    };

                    // TODO: dedupe
                    match default_track_selections {
                        Some(track_selections) => {
                            // 2a. Media is good to go, enqueue it
                            preparing_queue.push_back((media_identifier, track_selections));
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
                                        "Couldn't determine tracks for media {media_identifier:#?}. Ignoring it. {err}"
                                    );
                                    continue;
                                }
                            };

                            info!("Awaiting track selection to prepare {media_identifier:#?}.");
                            track_selection_wait_queue.push_back(domain::TrackSelectionItem {
                                media: media_identifier,
                                tracks,
                            });
                        }
                    }
                }
                PrepareMessage::SelectTracks(domain::TrackSelectionItem {
                    media: user_provided_media,
                    tracks,
                }) => {
                    // 1. Get the right item from our internal list
                    let Some((index, item)) = track_selection_wait_queue
                        .iter()
                        .enumerate()
                        .find(|(_, current)| current.media.id() == user_provided_media.id())
                    else {
                        error!(
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

                    // 3. If all good, put the item to queue and remove it from pending list
                    // TODO: dedupe
                    preparing_queue.push_back((media.clone(), selections));
                    track_selection_wait_queue.remove(index);
                }
                PrepareMessage::Done(media_identifier) => {
                    // 1. Last task was done, ready to run next item
                    task = None;

                    // 2. Tell media lib to recrawl (this may lead to an infinite loop)
                    // TODO implement loop detection
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

                    // 3. Remove from queue
                    match preparing_queue
                        .front()
                        .map(|(first_id, _)| first_id == &media_identifier)
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
                }
            }

            // 2. Announce we've updated the tasks
            if signal_receiver
                .updater
                .send((
                    preparing_queue
                        .iter()
                        .cloned()
                        .map(|(id, _)| id)
                        .flat_map(|id| id.strip_prefix(&media_dir))
                        .collect(),
                    track_selection_wait_queue
                        .iter()
                        .cloned()
                        .flat_map(|item| item.strip_prefix(&media_dir))
                        .collect(),
                ))
                .is_err()
            {
                error!("Processing list receiver was dropped. Can't update the processing list");
            }

            // 3. Start working on a task if none present
            if task.is_none()
                && let Some((head_id, head_track_selections)) = preparing_queue.front().cloned()
            {
                let sender = preparing_list_watcher.signal_sender.clone();
                task = Some(tokio::spawn(async move {
                    prepare(sender, head_id, head_track_selections).await;
                }))
            }
        }
    })
}

async fn prepare(
    sender: tokio::sync::mpsc::Sender<PrepareMessage>,
    identifier: domain::MediaIdentifier,
    track_selections: impl IntoIterator<Item = ffmpeg::TrackSelection>,
) {
    info!("Preparing {identifier:#?}");

    if let Err(err) = crate::prepare::prepare_media(&identifier, track_selections).await {
        error!("Couldn't prepare media with id {identifier:#?}. {err}");
    }

    if let Err(err) = sender.send(PrepareMessage::Done(identifier.clone())).await {
        error!("Prepared {identifier:#?} but couldn't tell prepare service about it. {err}");
    }
}
