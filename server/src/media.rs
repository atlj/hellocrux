use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, OpenOptions};
use std::io::Read;
use std::path::PathBuf;

use domain::{Media, MediaContent, MediaMetaData};
use log::{error, info, warn};

const MOVIE_EXTENSIONS: [&str; 2] = ["mov", "mp4"];

type Season = HashMap<u32, String>;
type Series = HashMap<u32, Season>;

pub async fn get_media_items(media_dir: PathBuf) -> Vec<Media> {
    let media_dir_contents = if let Ok(dir_contents) = fs::read_dir(media_dir.clone()) {
        dir_contents
    } else {
        error!("{:?} doesn't point to a valid media directory.", media_dir);
        return Vec::with_capacity(0);
    };

    return media_dir_contents
        .flatten()
        .flat_map(|entry| get_media_item(entry, &media_dir))
        .collect();
}

fn get_media_item(dir_entry: DirEntry, root_path: &PathBuf) -> Option<Media> {
    let entry_metadata = dir_entry.metadata().ok()?;

    if entry_metadata.is_file() {
        return None;
    }

    let mut metadata: Option<MediaMetaData> = None;
    let mut movie_file: Option<PathBuf> = None;
    let mut series: Series = HashMap::new();

    let read_dir = fs::read_dir(dir_entry.path()).ok()?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        let entry_metadata = if let Ok(entry_metadata) = entry.metadata() {
            entry_metadata
        } else {
            continue;
        };

        if entry_metadata.is_file() {
            let is_media_file = path
                .extension()
                .map(|extension| {
                    MOVIE_EXTENSIONS
                        .iter()
                        .find(|movie_extension| **movie_extension == extension)
                        .is_some()
                })
                .unwrap_or(false);

            if is_media_file {
                movie_file = Some(path.strip_prefix(root_path).unwrap().to_path_buf());
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

        if let Some((season_number, season)) = get_season(&root_path, &path) {
            series.insert(season_number, season);
        }
    }

    let unwrapped_metadata = if let Some(metadata) = metadata {
        metadata
    } else {
        warn!("No metadata found for: {:#?}", dir_entry.path());
        return None;
    };

    if let Some(movie_path) = movie_file {
        return Some(Media {
            id: dir_entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            metadata: unwrapped_metadata,
            content: MediaContent::Movie(movie_path.to_string_lossy().to_string()),
        });
    }

    if !series.is_empty() {
        return Some(Media {
            id: dir_entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            metadata: unwrapped_metadata,
            content: MediaContent::Series(series),
        });
    }

    None
}

fn get_season(root_path: &PathBuf, path: &PathBuf) -> Option<(u32, Season)> {
    let season_title_string = path.file_name()?;
    let season_number = get_numeric_content(season_title_string.to_str()?)?;
    let mut season: Season = HashMap::with_capacity(5);

    let contents = fs::read_dir(path).ok()?;

    for content in contents.flatten() {
        let content_path = content.path();
        let metadata = content.metadata().unwrap();

        if !metadata.is_file() {
            continue;
        }

        let is_media_file = content_path
            .extension()
            .map(|extension| {
                MOVIE_EXTENSIONS
                    .iter()
                    .find(|movie_extension| **movie_extension == extension)
                    .is_some()
            })
            .is_some();

        if is_media_file {
            if let Some(episode_number) =
                get_numeric_content(content_path.file_name().unwrap().to_str().unwrap())
            {
                season.insert(
                    episode_number,
                    content_path
                        .strip_prefix(root_path)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
    }

    if season.is_empty() {
        return None;
    }

    return Some((season_number, season));
}

pub fn get_numeric_content(string: &str) -> Option<u32> {
    let mut digits = Vec::<u32>::with_capacity(4);
    let mut peekable = string.chars().peekable();

    while let Some(char) = peekable.next() {
        if let Some(digit) = char.to_digit(10) {
            digits.push(digit);

            if let Some(next) = peekable.peek() {
                if !next.is_ascii_digit() {
                    break;
                }
            }
        }
    }

    if digits.is_empty() {
        return None;
    }

    Some(
        digits
            .into_iter()
            .rev()
            .enumerate()
            .rfold(0, |curr, (power, digit)| {
                curr + (digit * 10_u32.pow(power.try_into().unwrap()))
            }),
    )
}

#[cfg(test)]
mod test {
    use crate::media::get_numeric_content;

    #[test]
    fn can_get_numeric_content() {
        assert_eq!(get_numeric_content("1Ambush.mov"), Some(1));
        assert_eq!(get_numeric_content("176hey.exe"), Some(176));
        assert_eq!(get_numeric_content("02Ambush.mov"), Some(2));
        assert_eq!(get_numeric_content("22ey17.exe"), Some(22));
        assert_eq!(get_numeric_content("eyslkvjsdlkj03k.exe"), Some(3));
        assert_eq!(get_numeric_content("1"), Some(1));
        assert_eq!(get_numeric_content("Ambush.mov"), None);
    }
}
