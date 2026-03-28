use std::{ffi::OsStr, process::Stdio};

/// Run `ffmpeg`
///
/// Spawns an `ffmpeg` process and drives it to completion.
///
/// # Errors
/// Spawning a process can cause a lot of issues such as command not existing, status code being a
/// non-zero value etc.
pub(super) async fn ffmpeg(
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> crate::Result<String> {
    let result = tokio::process::Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|err| crate::Error::CouldntSpawn(err.to_string()))?;

    if !result.status.success() {
        let message: String = result
            .stdout
            .into_iter()
            .chain(std::iter::once(b'\n'))
            .chain(result.stderr)
            .map(|byte| byte as char)
            .collect();

        return Err(crate::Error::NonZeroExit(message));
    }

    Ok(result.stdout.into_iter().map(|byte| byte as char).collect())
}

/// Run `ffprobe`
///
/// Spawns an `ffprobe` process and drives it to completion.
///
/// # Errors
/// Spawning a process can cause a lot of issues such as command not existing, status code being a
/// non-zero value etc.
pub(super) async fn ffprobe(
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> crate::Result<String> {
    let result = tokio::process::Command::new("ffprobe")
        .args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|err| crate::Error::CouldntSpawn(err.to_string()))?;

    if !result.status.success() {
        let message: String = result
            .stdout
            .into_iter()
            .chain(std::iter::once(b'\n'))
            .chain(result.stderr)
            .map(|byte| byte as char)
            .collect();

        return Err(crate::Error::NonZeroExit(message));
    }

    Ok(result.stdout.into_iter().map(|byte| byte as char).collect())
}
