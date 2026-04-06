mod encode;
mod extract;
mod spawn;
mod track;

pub use encode::{TrackExt, TrackSelection, encode_video};

pub use extract::extract_tracks;
pub use track::get_tracks;

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
    #[error("Couldn't get tracks: '{0}'")]
    CouldntGetTracks(track::dto::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
