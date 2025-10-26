use std::time::Duration;

use torrent::{
    TorrentInfo,
    qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage},
};

#[tokio::test]
async fn test_event_loop() {
    // 1. Generate client
    let client = QBittorrentClient::try_new(None).unwrap();

    // 2. Spawn event loop
    let (torrent_list_sender, torrent_list_receiver): (
        tokio::sync::watch::Sender<Box<[TorrentInfo]>>,
        _,
    ) = tokio::sync::watch::channel(Box::new([]));
    let (torrent_event_loop_sender, torrent_event_loop_receiver) = tokio::sync::mpsc::channel(100);

    let event_loop_handle = tokio::spawn(async move {
        client
            .event_loop(torrent_event_loop_receiver, torrent_list_sender)
            .await
            .unwrap();
    });

    // 3. Add new torrent
    {
        let (add_torrent_result_sender, add_torrent_result_receiver) =
            tokio::sync::oneshot::channel();
        torrent_event_loop_sender.send(QBittorrentClientMessage::AddTorrent { hash: "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent", result_sender: add_torrent_result_sender }).await.unwrap();

        add_torrent_result_receiver.await.unwrap().unwrap();
    }

    // 4. Wait a bit because QBittorrent doesn't immediately add the torrent.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 5. Ask client to update its torrent list
    {
        let (update_torrent_list_result_sender, update_torrent_list_result_receiver) =
            tokio::sync::oneshot::channel();

        torrent_event_loop_sender
            .send(QBittorrentClientMessage::UpdateTorrentList {
                result_sender: update_torrent_list_result_sender,
            })
            .await
            .unwrap();

        update_torrent_list_result_receiver.await.unwrap().unwrap();
    }

    // 6. Make sure the list is not empty
    let value = torrent_list_receiver.borrow();
    dbg!(&value);
    assert!(!value.is_empty());

    // 7. Clean up the event loop explicitly
    event_loop_handle.abort();
}
