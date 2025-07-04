use std::ffi::OsStr;
use std::fs::{self, DirEntry, OpenOptions};
use std::io::Read;
use std::path::PathBuf;

use domain::{Media, MediaContent, MediaMetaData};

const MOVIE_EXTENSIONS: [&str; 2] = ["mov", "mp4"];

pub async fn get_media_items(media_dir: PathBuf) -> Vec<Media> {
    let media_dir_contents = if let Ok(dir_contents) = fs::read_dir(media_dir.clone()) {
        dir_contents
    } else {
        println!("{:?} doesn't point to a valid media directory.", media_dir);
        return Vec::with_capacity(0);
    };

    return media_dir_contents
        .flatten()
        .flat_map(|entry| get_media_item(entry))
        .collect();
}

fn get_media_item(dir_entry: DirEntry) -> Option<Media> {
    let entry_metadata = dir_entry.metadata().ok()?;

    if entry_metadata.is_file() {
        return None;
    }

    let mut metadata: Option<MediaMetaData> = None;
    let mut movie_file: Option<PathBuf> = None;

    let read_dir = fs::read_dir(dir_entry.path()).ok()?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let entry_metadata = if let Ok(entry_metadata) = entry.metadata() {
            entry_metadata
        } else {
            continue;
        };

        if entry_metadata.is_file() {
            let extension = path.extension().unwrap();
            let is_media_file = MOVIE_EXTENSIONS
                .iter()
                .find(|movie_extension| **movie_extension == extension)
                .is_some();

            if is_media_file {
                movie_file = Some(path);
                continue;
            }

            if path.file_name() == Some(OsStr::new("meta.json")) {
                let mut file = OpenOptions::new().read(true).open(path.clone()).unwrap();

                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents).expect(&format!(
                    "Couldn't read json file at: {:#?}",
                    path.clone().to_str()
                ));

                metadata = serde_json::from_str::<MediaMetaData>(&file_contents).ok();
            }

            continue;
        }

        // It means this has to be a series
    }

    let unwrapped_metadata = if let Some(metadata) = metadata {
        metadata
    } else {
        println!("No metadata found for: {:#?}", dir_entry.path());
        return None;
    };

    match movie_file {
        None => None,
        Some(movie_path) => Some(Media {
            id: dir_entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            metadata: unwrapped_metadata,
            content: MediaContent::Movie(movie_path.to_string_lossy().to_string()),
        }),
    }
}
