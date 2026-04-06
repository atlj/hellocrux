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
// TODO refactor whole func
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
            match signal {
                PrepareMessage::Prepare(media_identifier) => {
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
                            preparing_queue.push_back((media_identifier, track_selections));
                        }
                        None => {
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
                PrepareMessage::SelectTracks(domain::TrackSelectionItem { media, tracks }) => {
                    let Some(index) =
                        track_selection_wait_queue
                            .iter()
                            .enumerate()
                            .find_map(|(idx, current)| {
                                if current.media.id() == media.id() {
                                    Some(idx)
                                } else {
                                    None
                                }
                            })
                    else {
                        error!(
                            "Provided track selection for {media:#?} but looks like tracks were already provided. Ignoring it."
                        );
                        continue;
                    };

                    let media = &track_selection_wait_queue
                        .get(index)
                        .expect("We already found the index")
                        .media;

                    let selections = tracks
                        .into_iter()
                        .map(|track| {
                            ffmpeg::TrackExt::into_selection(
                                track,
                                media.path().media.clone().into(),
                            )
                        })
                        .collect::<Vec<_>>();

                    // TODO: dedupe
                    preparing_queue.push_back((media.clone(), selections));
                    track_selection_wait_queue.remove(index);
                }
                PrepareMessage::Done(media_identifier) => {
                    // 1c. Last task was done, update task list
                    task = None;

                    // Tell media lib to recrawl
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
