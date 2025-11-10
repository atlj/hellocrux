use std::time::Duration;

use domain::MediaMetaData;
use torrent::{
    TorrentInfo,
    qbittorrent_client::{QBittorrentClient, QBittorrentClientMessage},
};

#[tokio::test]
async fn test_event_loop() {
    // 1. Generate client
    let client = QBittorrentClient::try_new(None).unwrap();

    // 2. Remove what's on profile dir
    tokio::fs::remove_dir_all(&client.profile_dir)
        .await
        .unwrap();

    // 3. Spawn event loop
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

    let metadata = MediaMetaData {
        title: "My Movie".to_string(),
        thumbnail: "https://image.com".to_string(),
    };

    // 4. Try adding a faulty torrent
    {
        let (add_torrent_result_sender, add_torrent_result_receiver) =
            tokio::sync::oneshot::channel();
        torrent_event_loop_sender
            .send(QBittorrentClientMessage::AddTorrent {
                hash: "faulty-hash".into(),
                result_sender: add_torrent_result_sender,
                metadata: Box::new(metadata.clone()),
            })
            .await
            .unwrap();

        assert!(add_torrent_result_receiver.await.unwrap().is_err());
    }

    // 5. Add new torrent
    {
        let (add_torrent_result_sender, add_torrent_result_receiver) =
            tokio::sync::oneshot::channel();

        torrent_event_loop_sender.send(
            QBittorrentClientMessage::AddTorrent {
                hash: "https://cdimage.debian.org/debian-cd/current/arm64/bt-cd/debian-13.1.0-arm64-netinst.iso.torrent".into(),
                result_sender: add_torrent_result_sender,
                metadata: Box::new(metadata)
            }
         ).await.unwrap();

        add_torrent_result_receiver.await.unwrap().unwrap();
    }

    // 6. Wait a bit because QBittorrent doesn't immediately add the torrent.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 7. Ask client to update its torrent list
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

    // 8. Make sure the list is not empty
    let value = torrent_list_receiver.borrow();
    dbg!(&value);
    assert!(!value.is_empty());

    // 9. Remove torrent
    {
        let (update_torrent_list_result_sender, update_torrent_list_result_receiver) =
            tokio::sync::oneshot::channel();

        torrent_event_loop_sender
            .send(QBittorrentClientMessage::RemoveTorrent {
                id: value.first().unwrap().hash.clone(),
                result_sender: update_torrent_list_result_sender,
            })
            .await
            .unwrap();

        update_torrent_list_result_receiver.await.unwrap().unwrap();
    }

    // 10. Wait a bit because QBittorrent doesn't immediately remove a torrent.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 11. Ask client to update its torrent list
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

    // 12. Make sure the list is empty
    let value = torrent_list_receiver.borrow();
    dbg!(&value);
    assert!(value.is_empty());

    // 13. Clean up the event loop explicitly
    event_loop_handle.abort();
}
