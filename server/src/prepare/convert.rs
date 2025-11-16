use std::{path::Path, process::Stdio};

use domain::MediaStream;
use log::info;

pub async fn convert_media(input_path: &Path, output_path: &Path) -> super::Result<()> {
    // 1. Check for input file
    {
        let does_input_file_exist = tokio::fs::try_exists(input_path).await.map_err(|err| {
            super::Error::ConvertError(
                format!("Couldn't check if input file exists. Reason: {err}").into(),
            )
        })?;

        if !does_input_file_exist {
            return Err(super::Error::ConvertError(
                format!("Input file at {} doesn't exist", input_path.display()).into(),
            ));
        }
    }

    // 2. Create output dir if it doesn't exist
    {
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|err| {
                super::Error::ConvertError(
                    format!(
                        "Couldn't create output dir at {}. Reason: {err}",
                        output_path.display()
                    )
                    .into(),
                )
            })?;
        }
    }

    let override_tag: Option<&str> = {
        let video_codec = get_codec(input_path, &domain::MediaStream::Video).await?;
        match video_codec.as_ref() {
            // Override the tag for hevc because Apple expects it like that.
            // https://stackoverflow.com/questions/49128084/playing-h-265-video-file-using-avplayer
            "hevc" => Some("hvc1"),
            _ => None,
        }
    };

    let audio_codec: &str = {
        let audio_codec = get_codec(input_path, &domain::MediaStream::Audio).await?;
        match audio_codec.as_ref() {
            "aac" => "copy",
            _ => "aac",
        }
    };

    // 3. Run ffmpeg until it ends
    {
        let input_path_string = input_path.as_os_str().to_string_lossy();
        let output_path_string = output_path.as_os_str().to_string_lossy();
        let args = {
            let mut args = vec![
                "-i",
                &input_path_string,
                // Copy everything
                "-c:v",
                "copy",
                "-c:a",
                audio_codec,
            ];

            // Override the tag for hevc
            if let Some(tag) = override_tag {
                args.push("-tag:v");
                args.push(tag);
            }

            args.push(&output_path_string);
            args
        };

        let result = tokio::process::Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .output()
            .await
            .map_err(|err| {
                super::Error::ConvertError(format!("Couldn't spawn ffmpeg. Reason: {err}").into())
            })?;

        if !result.status.success() {
            return Err(super::Error::ConvertError(
                format!(
                    "ffmpeg exited with non-zero status. stdout: {:?}, stderr: {:?}",
                    result
                        .stdout
                        .into_iter()
                        .map(|byte| byte as char)
                        .collect::<String>(),
                    result
                        .stderr
                        .into_iter()
                        .map(|byte| byte as char)
                        .collect::<String>()
                )
                .into(),
            ));
        }
    }

    Ok(())
}

pub async fn get_codec(media_file: &Path, stream_type: &MediaStream) -> super::Result<String> {
    let stream_identifier = match stream_type {
        MediaStream::Video => 'v',
        MediaStream::Audio => 'a',
    };

    let result = tokio::process::Command::new("ffprobe")
        .args([
            // Error on empty
            "-v",
            "error",
            // Select the first video stream
            "-select_streams",
            &format!("{stream_identifier}:0"),
            // Show the codec name
            "-show_entries",
            "stream=codec_name",
            // Don't print anything unuseful
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &media_file.as_os_str().to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|err| {
            super::Error::ConvertError(format!("Couldn't spawn ffmpeg. Reason: {err}").into())
        })?;

    if !result.status.success() {
        return Err(super::Error::ConvertError(
            format!(
                "ffprobe exited with non-zero status. stdout: {:?}, stderr: {:?}",
                result
                    .stdout
                    .into_iter()
                    .map(|byte| byte as char)
                    .collect::<String>(),
                result
                    .stderr
                    .into_iter()
                    .map(|byte| byte as char)
                    .collect::<String>()
            )
            .into(),
        ));
    }

    Ok(result
        .stdout
        .into_iter()
        .flat_map(|byte| match byte as char {
            '\n' => None,
            _ => Some(byte as char),
        })
        .collect())
}

/// ## Panics
/// Panics when `media_file` has no extension or it's not a file
pub fn should_convert(media_file: &Path) -> bool {
    match media_file.extension() {
        Some(extension) => match extension.to_string_lossy().as_ref() {
            "mp4" | "hevc" | "mov" | "avi" | "ts" => false,
            "mkv" => true,
            _ => {
                info!(
                    "Found a file with a potentially unsupported format at {} while trying to convert it. Trying to convert it anyway.",
                    media_file.display()
                );
                true
            }
        },
        None => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::prepare::convert::{convert_media, get_codec};

    #[tokio::test]
    async fn test_convert_file_to_mov() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp/test.mov")).await;

        convert_media(
            &test_data_path.join("test.mkv"),
            &test_data_path.join("tmp/test.mov"),
        )
        .await
        .unwrap();

        assert!(
            tokio::fs::try_exists(test_data_path.join("tmp/test.mov"))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_codec() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();
        assert_eq!(
            get_codec(
                &test_data_path.join("test.mkv"),
                &domain::MediaStream::Video
            )
            .await
            .unwrap(),
            "h264".to_string()
        );
        assert_eq!(
            get_codec(
                &test_data_path.join("moonlight_sonata.ogg"),
                &domain::MediaStream::Audio
            )
            .await
            .unwrap(),
            "vorbis".to_string()
        );

        assert_eq!(
            get_codec(
                &test_data_path.join("test.h265.mkv"),
                &domain::MediaStream::Video
            )
            .await
            .unwrap(),
            "hevc".to_string()
        );

        assert!(
            get_codec(
                &test_data_path.join("broken.mkv"),
                &domain::MediaStream::Video
            )
            .await
            .is_err()
        );
    }
}
