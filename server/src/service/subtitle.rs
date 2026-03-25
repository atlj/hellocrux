use std::path::{Path, PathBuf};

use domain::{
    language::LanguageCode,
    subtitles::{SubtitleDownloadError, SubtitleProvider, SubtitleSelection},
};
use log::{info, warn};
use open_subtitles::OpenSubtitlesClient;
use tokio::io::AsyncWriteExt;

pub enum SubtitleSignal {
    Download {
        media_path: PathBuf,
        selection: SubtitleSelection,
        result_sender: tokio::sync::oneshot::Sender<Result<(), SubtitleDownloadError>>,
        language_code: LanguageCode,
    },
}

pub type SubtitleSignalSender = tokio::sync::mpsc::Sender<SubtitleSignal>;
pub type SubtitleSignalReceiver = tokio::sync::mpsc::Receiver<SubtitleSignal>;

/// A service that downloads and manages subtitles
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
                    selection,
                    result_sender,
                    language_code,
                } => {
                    let result = download_subtitle(
                        &media_path,
                        &subtitle_provider,
                        selection,
                        language_code,
                    )
                    .await;
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
    selection: SubtitleSelection,
    language_code: LanguageCode,
) -> Result<(), SubtitleDownloadError> {
    let id = selection.subtitle_id();
    let subtitles_folder_path = media_path.with_file_name("subtitles");

    let subtitle_path = get_subtitle_path(&subtitles_folder_path, &selection, &language_code);

    info!("Downloading subtitle with id {} to {subtitle_path:#?}", id);

    // 1. Check if subtitle already exists early so user doesn't use quota
    if tokio::fs::try_exists(&subtitle_path)
        .await
        .map_err(|_| SubtitleDownloadError::InternalFileSystemError)?
    {
        warn!("Subtitle with id {} already exists. Skipping it", id);
        return Err(SubtitleDownloadError::SubtitleAlreadyExists);
    }

    // 2. Download the string data
    let subtitle_string = subtitle_provider.download(id).await.map_err(|_|
            // TODO: Check other types of errors too
            SubtitleDownloadError::DownloadQuotaReached)?;

    // 3. Write the string data
    {
        tokio::fs::create_dir_all(subtitles_folder_path)
            .await
            .map_err(|_| SubtitleDownloadError::InternalFileSystemError)?;

        let mut subtitle_file = tokio::fs::OpenOptions::new()
            // Prevents time of check vs time of use bug
            .create_new(true)
            .write(true)
            .open(&subtitle_path)
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

    info!("Added subtitle with id {} to {subtitle_path:#?}", id);

    Ok(())
}

fn get_subtitle_path(
    subtitles_folder: impl AsRef<Path>,
    selection: &SubtitleSelection,
    language_code: &LanguageCode,
) -> PathBuf {
    let file_name = match selection {
        SubtitleSelection::Series {
            subtitle_id,
            episode_identifier,
        } => format!(
            "{}-{}-{}.srt",
            episode_identifier.episode_no,
            language_code.to_iso639_2t(),
            subtitle_id
        ),
        SubtitleSelection::Movie { subtitle_id } => {
            format!("{}-{}.srt", language_code.to_iso639_2t(), subtitle_id)
        }
    };

    subtitles_folder.as_ref().join(file_name)
}
