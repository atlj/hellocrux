use std::{path::Path, process::Stdio};

async fn convert_file_to_mp4(input_path: &Path, output_path: &Path) -> super::Result<()> {
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

    // 3. Run ffmpeg until it ends
    {
        let result = tokio::process::Command::new("ffmpeg")
            .args([
                "-i",
                &input_path.as_os_str().to_string_lossy(),
                // Copy video
                "-c:v",
                "copy",
                // Copy audio
                "-c:a",
                "copy",
                &output_path.as_os_str().to_string_lossy(),
            ])
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .output()
            .await
            .map_err(|err| {
                super::Error::ConvertError(format!("Couldn't spawn ffmpeg. Reason: {err}").into())
            })?;

        dbg!(&result.stdout, &result.stderr, &result.status);

        if !result.status.success() {
            return Err(super::Error::ConvertError(
                format!(
                    "ffmpeg exited with non-zero status. stdout: {:?}, stderr: {:?}",
                    result.stdout.as_slice(),
                    result.stderr.as_slice()
                )
                .into(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::prepare::convert::convert_file_to_mp4;

    #[tokio::test]
    async fn test_convert_file_to_mp4() {
        let test_data_path: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/test-data").into();

        let _ = tokio::fs::remove_dir_all(test_data_path.join("tmp")).await;

        convert_file_to_mp4(
            &test_data_path.join("test.mkv"),
            &test_data_path.join("tmp/test.mp4"),
        )
        .await
        .unwrap();

        assert!(
            tokio::fs::try_exists(test_data_path.join("tmp/test.mp4"))
                .await
                .unwrap()
        );
    }
}
