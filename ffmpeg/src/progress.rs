use std::{str::FromStr, time::Duration};

#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    pub frame: usize,
    pub fps: f64,
    pub bitrate: String,
    pub total_size_bytes: usize,
    // TODO parse from millis
    pub out_time: Duration,
    pub dup_frames: usize,
    pub drop_frames: usize,
    // TODO parse from string
    pub speed: f64,
    pub progress: ProgressState,
}

impl FromStr for Progress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key_value_lines = s.split('\n');

        let mut frame: Option<usize> = None;
        let mut fps: Option<f64> = None;
        let mut bitrate: Option<String> = None;
        let mut total_size_bytes: Option<usize> = None;
        let mut out_time: Option<Duration> = None;
        let mut dup_frames: Option<usize> = None;
        let mut drop_frames: Option<usize> = None;
        let mut speed: Option<f64> = None;
        let mut progress: Option<ProgressState> = None;

        for line in key_value_lines {
            let Some((key, value)) = line.split_once('=') else {
                return Err(Error::InvalidLine(line.to_string()));
            };

            if value == "N/A" {
                return Err(Error::IncompleteValue(key.to_string()));
            }

            match key {
                "frame" => {
                    frame = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "frame".to_string(),
                        value: value.to_string(),
                    })?);
                }
                "fps" => {
                    fps = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "fps".to_string(),
                        value: value.to_string(),
                    })?);
                }
                "bitrate" => {
                    bitrate = Some(
                        value
                            .chars()
                            .skip_while(|char| char.is_whitespace())
                            .collect(),
                    );
                }
                "total_size" => {
                    total_size_bytes = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "total_size_bytes".to_string(),
                        value: value.to_string(),
                    })?);
                }
                "out_time_ms" | "out_time_us" => {
                    let value_usize: usize = value.parse().map_err(|_| Error::InvalidValue {
                        key: "out_time".to_string(),
                        value: value.to_string(),
                    })?;

                    out_time = Some(Duration::from_micros(value_usize as u64));
                }
                "dup_frames" => {
                    dup_frames = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "dup_frames".to_string(),
                        value: value.to_string(),
                    })?);
                }
                "drop_frames" => {
                    drop_frames = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "drop_frames".to_string(),
                        value: value.to_string(),
                    })?);
                }
                "speed" => {
                    let (value_numeric_part, multiplier) = value.split_at(value.len() - 1);
                    if multiplier != "x" {
                        return Err(Error::InvalidValue {
                            key: "speed".to_string(),
                            value: value.to_string(),
                        });
                    }

                    speed = Some(
                        value_numeric_part
                            .parse()
                            .map_err(|_| Error::InvalidValue {
                                key: "speed".to_string(),
                                value: value.to_string(),
                            })?,
                    );
                }
                "progress" => {
                    progress = Some(value.parse().map_err(|_| Error::InvalidValue {
                        key: "progress".to_string(),
                        value: value.to_string(),
                    })?);
                }

                _ => {}
            }
        }

        let Some(frame) = frame else {
            return Err(Error::MissingKey("frame".to_string()));
        };
        let Some(fps) = fps else {
            return Err(Error::MissingKey("fps".to_string()));
        };
        let Some(bitrate) = bitrate else {
            return Err(Error::MissingKey("bitrate".to_string()));
        };
        let Some(total_size_bytes) = total_size_bytes else {
            return Err(Error::MissingKey("total_size_bytes".to_string()));
        };
        let Some(out_time) = out_time else {
            return Err(Error::MissingKey("out_time".to_string()));
        };
        let Some(dup_frames) = dup_frames else {
            return Err(Error::MissingKey("dup_frames".to_string()));
        };
        let Some(drop_frames) = drop_frames else {
            return Err(Error::MissingKey("drop_frames".to_string()));
        };
        let Some(speed) = speed else {
            return Err(Error::MissingKey("speed".to_string()));
        };
        let Some(progress) = progress else {
            return Err(Error::MissingKey("progress".to_string()));
        };

        Ok(Progress {
            frame,
            fps,
            bitrate,
            total_size_bytes,
            out_time,
            dup_frames,
            drop_frames,
            speed,
            progress,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressState {
    Continue,
    End,
}

impl FromStr for ProgressState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "continue" => Ok(Self::Continue),
            "end" => Ok(Self::End),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Value has N/A for key {0}")]
    IncompleteValue(String),
    #[error("Line is invalid ({0})")]
    InvalidLine(String),
    #[error("Invalid value ({value}) passed for {key}")]
    InvalidValue { key: String, value: String },
    #[error("Value is missing key $0")]
    MissingKey(String),
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Duration};

    use crate::progress::{Progress, ProgressState};

    const FIRST_LINE: &str = "frame=0
fps=0.00
stream_0_0_q=0.0
bitrate=N/A
total_size=0
out_time_us=N/A
out_time_ms=N/A
out_time=N/A
dup_frames=0
drop_frames=0
speed=N/A
progress=continue";

    #[test]
    fn test_parse_incomplete_line() {
        assert_eq!(
            Progress::from_str(FIRST_LINE),
            Err(super::Error::IncompleteValue("bitrate".to_string()))
        );
    }

    const CONTINUE_LINE: &str = "frame=943
fps=35.57
stream_0_0_q=36.0
bitrate=   0.0kbits/s
total_size=44
out_time_us=37640000
out_time_ms=37640000
out_time=00:00:37.640000
dup_frames=0
drop_frames=0
speed=1.42x
progress=continue";

    #[test]
    fn test_parse_continue_line() {
        assert_eq!(
            Progress::from_str(CONTINUE_LINE),
            Ok(Progress {
                frame: 943,
                fps: 35.57,
                bitrate: "0.0kbits/s".to_string(),
                total_size_bytes: 44,
                out_time: Duration::from_micros(37640000),
                dup_frames: 0,
                drop_frames: 0,
                speed: 1.42,
                progress: ProgressState::Continue
            })
        );
    }

    const END_LINE: &str = "frame=12500
fps=36.48
stream_0_0_q=36.0
bitrate=  23.8kbits/s
total_size=1485771
out_time_us=499920000
out_time_ms=499920000
out_time=00:08:19.920000
dup_frames=0
drop_frames=0
speed=1.46x
progress=end";

    #[test]
    fn test_parse_end_line() {
        assert_eq!(
            Progress::from_str(END_LINE),
            Ok(Progress {
                frame: 12500,
                fps: 36.48,
                bitrate: "23.8kbits/s".to_string(),
                total_size_bytes: 1485771,
                out_time: Duration::from_micros(499920000),
                dup_frames: 0,
                drop_frames: 0,
                speed: 1.46,
                progress: ProgressState::End
            })
        );
    }

    use crate::spawn::ffmpeg;

    #[tokio::test]
    #[ignore = "this is creating a non temporary file"]
    async fn test_get_progress() {
        let _ = env_logger::builder().is_test(true).try_init();
        let (progress_sender, progress_receiver) = tokio::sync::watch::channel(None);

        ffmpeg(
            [
                "-f", "lavfi", "-i", "testsrc", "-map", "0:0", "-c:v", "libx265", "-t", "10", "-y",
                // TODO use a temp dir
                "test.mp4",
            ]
            .into_iter(),
            Some(progress_sender),
        )
        .await
        .unwrap();

        dbg!(progress_receiver.borrow());
        assert!(progress_receiver.borrow().is_some());
    }
}
