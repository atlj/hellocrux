use std::path::{Path, PathBuf};

use domain::{
    series::EpisodeIdentifier,
    subtitles::{SubtitleDownloadError, SubtitleRequest},
};
use subtitles::{OpenSubtitlesClient, SubtitleProvider};
use tokio::io::AsyncWriteExt;

pub enum SubtitleSignal {
    Download {
        media_path: PathBuf,
        request: SubtitleRequest,
        result_sender: tokio::sync::oneshot::Sender<Result<(), SubtitleDownloadError>>,
    },
}

pub type SubtitleSignalSender = tokio::sync::mpsc::Sender<SubtitleSignal>;
pub type SubtitleSignalReceiver = tokio::sync::mpsc::Receiver<SubtitleSignal>;

/// A service that downlaods and manages subtitles
pub fn spawn(
    mut receiver: SubtitleSignalReceiver,
    subtitle_provider: OpenSubtitlesClient,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let signal = receiver
                .recv()
                .await
                .expect("Subtitle sender was dropped early");

            match signal {
                SubtitleSignal::Download {
                    media_path,
                    request,
                    result_sender,
                } => {
                    let result = download_subtitle(&media_path, &subtitle_provider, request).await;
                    // We don't care if the sender is still listening
                    let _ = result_sender.send(result);
                }
            }
        }
    })
}

async fn download_subtitle(
    media_path: &Path,
    subtitle_provider: &OpenSubtitlesClient,
    request: SubtitleRequest,
) -> Result<(), SubtitleDownloadError> {
    let subtitles_folder_path = media_path.with_file_name("subtitles");
    let subtitle_path = match request.episode_identifier {
        Some(EpisodeIdentifier { episode_no, .. }) => {
            subtitles_folder_path.join(format!("{}-{}.srt", episode_no, request.subtitle_id))
        }
        None => subtitles_folder_path.join(format!("{}.srt", request.subtitle_id)),
    };

    // 1. Check if subtitle already exists early so user doesn't use quota
    if tokio::fs::try_exists(&subtitle_path)
        .await
        .map_err(|_| SubtitleDownloadError::InternalFileSystemError)?
    {
        return Err(SubtitleDownloadError::SubtitleAlreadyExists);
    }

    // 2. Download the string data
    let subtitle_string = subtitle_provider
        .download(&request.subtitle_id)
        .await
        .map_err(|_| 
            // TODO: Check other types of errors too
            SubtitleDownloadError::DownloadQuotaReached)?;

    // 3. Write the string data
    {
        tokio::fs::create_dir_all(subtitles_folder_path).await.map_err(|_|SubtitleDownloadError::InternalFileSystemError)?;

        let mut subtitle_file = tokio::fs::OpenOptions::new()
            // Prevents time of check vs time of use bug
            .create_new(true)
            .write(true)
            .open(subtitle_path)
            .await
            .map_err(|_| SubtitleDownloadError::SubtitleAlreadyExists)?;

        subtitle_file
            .write(subtitle_string.as_bytes())
            .await
            .map_err(|_| SubtitleDownloadError::InternalFileSystemError)?;

        subtitle_file
            .flush()
            .await
            .map_err(|_| SubtitleDownloadError::InternalFileSystemError)?;
    }

    Ok(())
}
