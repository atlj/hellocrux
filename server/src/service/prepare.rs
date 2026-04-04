use std::collections::VecDeque;

use log::error;

pub enum PrepareMessage {
    Prepare(domain::MediaIdentifier),
}

pub type PreparingListWatcher =
    crate::signal::SignalWatcher<PrepareMessage, Vec<domain::MediaIdentifier>>;
pub type PreparingListReceiver =
    crate::signal::SignalReceiver<PrepareMessage, Vec<domain::MediaIdentifier>>;

/// Makes media files compatible
pub async fn spawn(
    mut signal_receiver: PreparingListReceiver,
    crate::AppState {
        media_signal_watcher,
        preparing_list_watcher,
        ..
    }: crate::AppState,
) -> tokio::task::JoinHandle<()> {
    let mut preparing_list = VecDeque::with_capacity(50);
    let handle = tokio::spawn(async move {
        while let Some(signal) = signal_receiver.signal_receiver.recv().await {
            match signal {
                PrepareMessage::Prepare(media_identifier) => {
                    preparing_list.push_back(media_identifier);

                    if signal_receiver
                        .updater
                        .send(preparing_list.clone().into())
                        .is_err()
                    {
                        error!("Media list receiver was dropped. Can't update the media library")
                    }
                }
            }
        }
    });

    handle
}
