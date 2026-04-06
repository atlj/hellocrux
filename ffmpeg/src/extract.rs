pub async fn extract_tracks(
    media_path: String,
    track_mapping: Vec<(usize, String)>,
) -> super::Result<()> {
    let input_args = ["-i".to_string(), media_path];
    let mapping_args = track_mapping.into_iter().flat_map(|(id, output)| {
        [
            // Select the track
            "-map".to_string(),
            format!("0:{id}"),
            // Use the `text` codec
            "-c:s".to_string(),
            "text".to_string(),
            // Output
            output,
        ]
    });

    let args = input_args.into_iter().chain(mapping_args);

    crate::spawn::ffmpeg(args).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::extract_tracks;

    fn fixtures_path() -> PathBuf {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into()
    }

    #[tokio::test]
    async fn extract_subtitles() {
        let dir = tempfile::tempdir().unwrap();
        let input = fixtures_path().join("h264_aac_3subs.mp4");
        let output = dir.path().join("hey.srt").to_string_lossy().to_string();

        extract_tracks(
            input.to_string_lossy().to_string(),
            vec![(3, output.clone())],
        )
        .await
        .unwrap();

        let exists = tokio::fs::try_exists(&output).await.unwrap();
        assert!(exists);
    }
}
