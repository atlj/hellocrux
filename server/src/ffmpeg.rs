use std::{ffi::OsStr, process::Stdio};

pub async fn ffmpeg(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String, Error> {
    let result = tokio::process::Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|err| Error::CouldntSpawn(err.to_string()))?;

    if !result.status.success() {
        let message: String = result
            .stdout
            .into_iter()
            .chain(std::iter::once('\n' as u8))
            .chain(result.stderr)
            .map(|byte| byte as char)
            .collect();

        return Err(Error::NonZeroExit(message));
    }

    Ok(result.stdout.into_iter().map(|byte| byte as char).collect())
}

pub async fn ffprobe(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String, Error> {
    let result = tokio::process::Command::new("ffprobe")
        .args(args)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|err| Error::CouldntSpawn(err.to_string()))?;

    if !result.status.success() {
        let message: String = result
            .stdout
            .into_iter()
            .chain(std::iter::once('\n' as u8))
            .chain(result.stderr)
            .map(|byte| byte as char)
            .collect();

        return Err(Error::NonZeroExit(message));
    }

    Ok(result.stdout.into_iter().map(|byte| byte as char).collect())
}

#[derive(Debug)]
pub enum Error {
    CouldntSpawn(String),
    NonZeroExit(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl std::error::Error for Error {}
