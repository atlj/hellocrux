mod spawn;
mod track;

pub use track::{Track, get_tracks};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't spawn ffmpeg due to {0}")]
    CouldntSpawn(String),
    #[error("ffmpeg exited with a non-zero status: {0}")]
    NonZeroExit(String),
    #[error("ffmpeg didn't produce any outputs")]
    MissingOutput,
}

pub type Result<T> = core::result::Result<T, Error>;
