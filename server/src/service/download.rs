use torrent::{
    TorrentInfo,
    qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage},
};

pub type DownloadSignalWatcher =
    crate::signal::SignalWatcher<QBittorrentClientMessage, Box<[TorrentInfo]>>;
pub type DownloadSignalReceiver =
    crate::signal::SignalReceiver<QBittorrentClientMessage, Box<[TorrentInfo]>>;

/// A service that handles downloading
pub async fn spawn(
    download_path: std::path::PathBuf,
    download_signal_receiver: DownloadSignalReceiver,
    download_signal_watcher: DownloadSignalWatcher,
) -> tokio::task::JoinHandle<()> {
    let client = QBittorrentClient::try_new(Some(download_path)).unwrap();

    let handle = tokio::spawn(async move {
        client
            .event_loop(
                download_signal_receiver.signal_receiver,
                download_signal_receiver.updater,
            )
            .await
            .expect("Event loop exited sooner than expected");
    });

    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    download_signal_watcher
        .signal_sender
        .send(QBittorrentClientMessage::UpdateTorrentList { result_sender })
        .await
        .unwrap();

    result_receiver.await.unwrap().unwrap();

    handle
}
