use std::path::{Path, PathBuf};

use domain::subtitles::{SubtitleDownloadError, SubtitleRequest};
use subtitles::OpenSubtitlesClient;

pub enum SubtitleSignal {
    Download {
        media_id: String,
        request: SubtitleRequest,
        result_sender: tokio::sync::oneshot::Sender<Result<(), SubtitleDownloadError>>,
    },
}

pub type SubtitleSignalSender = tokio::sync::mpsc::Sender<SubtitleSignal>;
pub type SubtitleSignalReceiver = tokio::sync::mpsc::Receiver<SubtitleSignal>;

/// A service that downlaods and manages subtitles
pub fn spawn(
    media_dir: PathBuf,
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
                    media_id,
                    request,
                    result_sender,
                } => {
                    let result =
                        download_subtitle(&media_dir, &subtitle_provider, &media_id, request).await;
                    // We don't care if the sender is still listening
                    let _ = result_sender.send(result);
                }
            }
        }
    })
}

async fn download_subtitle(
    _media_dir: &Path,
    _subtitle_provider: &OpenSubtitlesClient,
    _media_id: &str,
    _request: SubtitleRequest,
) -> Result<(), SubtitleDownloadError> {
    // 1. Check if destination exists
    todo!()
}
