use log::{debug, error};
use std::{ffi::OsStr, os::fd::AsRawFd, process::Stdio, str::FromStr};
use tokio::io::AsyncBufReadExt;

/// Run `ffmpeg`
///
/// Spawns an `ffmpeg` process and drives it to completion.
///
/// # Errors
/// Spawning a process can cause a lot of issues such as command not existing, status code being a
/// non-zero value etc.
pub(super) async fn ffmpeg<Args, Arg>(
    args: Args,
    progress_sender: Option<tokio::sync::watch::Sender<Option<crate::progress::Progress>>>,
) -> crate::Result<String>
where
    Arg: AsRef<OsStr>,
    Args: IntoIterator<Item = Arg> + std::fmt::Debug,
{
    debug!("Calling ffmpeg with: {args:#?}");

    let mut command = tokio::process::Command::new("ffmpeg");

    let (process_observer_handle, pipe_writer) = match progress_sender {
        Some(progress_sender) => {
            let (pipe_reader, pipe_writer) =
                std::io::pipe().map_err(crate::Error::CantCreatePipe)?;
            let pipe_fd = pipe_writer.as_raw_fd();

            command.arg("-progress").arg(format!("pipe:{pipe_fd}"));

            unsafe {
                make_fd_available_in_command(&mut command, pipe_fd);
            }

            (
                Some(tokio::spawn(async move {
                    observe_progress(pipe_reader, progress_sender).await;
                })),
                Some(pipe_writer),
            )
        }
        None => (None, None),
    };

    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let result = command
        .spawn()
        .map_err(|err| crate::Error::CouldntSpawn(err.to_string()))?
        .wait_with_output()
        .await
        .map_err(|err| crate::Error::CouldntSpawn(err.to_string()))?;

    drop(pipe_writer);
    if let Some(handle) = process_observer_handle {
        let _ = handle.await;
    }

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
    args: impl IntoIterator<Item = impl AsRef<OsStr>> + std::fmt::Debug,
) -> crate::Result<String> {
    debug!("Calling ffprobe with: {args:#?}");

    let result = tokio::process::Command::new("ffprobe")
        .args(args)
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

async fn observe_progress(
    pipe_reader: std::io::PipeReader,
    progress_sender: tokio::sync::watch::Sender<Option<crate::progress::Progress>>,
) {
    let pipe_receiver = match tokio::net::unix::pipe::Receiver::from_owned_fd(
        std::os::fd::OwnedFd::from(pipe_reader),
    ) {
        Ok(pipe_receiver) => pipe_receiver,
        Err(err) => {
            error!("ffmpeg couldn't create a pipe to receive progress: {err}");
            return;
        }
    };

    let mut lines = tokio::io::BufReader::new(pipe_receiver).lines();

    let mut block = String::with_capacity(300);

    while let Ok(Some(line)) = lines.next_line().await {
        block.push_str(&line);
        if line.starts_with("progress=") {
            dbg!(&block);
            match crate::progress::Progress::from_str(&block) {
                Ok(progress) => {
                    dbg!(&progress);
                    if let Err(err) = progress_sender.send(Some(progress)) {
                        error!("ffmpeg couldn't update progress: {err}")
                    }
                }
                Err(crate::progress::Error::IncompleteValue(key)) => {
                    debug!("Ignoring ffmpeg progress for since {key} is set to N/A")
                }
                Err(err) => {
                    error!("ffmpeg couldn't parse progress: {err}")
                }
            };
            block.clear();
        } else {
            block.push('\n');
        }
    }
}

unsafe fn make_fd_available_in_command(command: &mut tokio::process::Command, fd: i32) {
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags == -1 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}
