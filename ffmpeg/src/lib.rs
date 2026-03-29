mod encode;
mod spawn;
mod track;

pub use encode::{EncodeOptions, TrackSelection, encode_video};

pub use track::{Track, get_tracks};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't spawn ffmpeg/ffprobe due to '{0}'")]
    CouldntSpawn(String),
    #[error("ffmpeg/ffprobe exited with a non-zero status: '{0}'")]
    NonZeroExit(String),
    #[error("ffmpeg/ffprobe didn't produce any outputs")]
    MissingOutput,
    #[error("ffmpeg/ffprobe produced unexpected output: '{0}'")]
    UnexpectedOutput(String),
}

pub type Result<T> = core::result::Result<T, Error>;
