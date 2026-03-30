use std::{
    fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use domain::series::{EditSeriesFileMappingForm, EpisodeIdentifier};

pub enum Fixture {
    Prepare,
    PrepareSeries,
}

impl Fixture {
    fn as_path(&self) -> &'static str {
        match self {
            Self::Prepare => "prepare",
            Self::PrepareSeries => "prepare_series",
        }
    }
}

pub fn fixtures_path() -> PathBuf {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures").into()
}

/// Copies the named fixture directory into a fresh `TempDir` under `source/`
/// and returns both. The `TempDir` must be kept alive for the duration of the test.
pub fn fixture_sandbox(fixture: Fixture) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source");
    copy_dir_all(fixtures_path().join(fixture.as_path()), &source).unwrap();
    (tmp, source)
}

/// Builds an `EditSeriesFileMappingForm` from a slice of `(file_path, season, episode)` tuples.
pub fn episode_mapping<T>(id: &str, entries: &[(&str, u32, u32)]) -> EditSeriesFileMappingForm<T> {
    let file_mapping = entries
        .iter()
        .map(|(path, season, episode)| {
            (
                path.to_string(),
                EpisodeIdentifier {
                    season_no: *season,
                    episode_no: *episode,
                },
            )
        })
        .collect();

    EditSeriesFileMappingForm {
        id: id.into(),
        file_mapping,
        phantom: PhantomData,
    }
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
