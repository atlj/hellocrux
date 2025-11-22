use std::path::Path;

pub fn is_supported_video_file(path: impl AsRef<Path>) -> bool {
    if !path.as_ref().is_file() {
        return false;
    }
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        None => false,
        Some(extension) => matches!(extension, "mp4" | "mov"),
    }
}
pub fn is_video_file(path: impl AsRef<Path>) -> bool {
    if !path.as_ref().is_file() {
        return false;
    }
    match path.as_ref().extension().and_then(|os_str| os_str.to_str()) {
        None => false,
        Some(extension) => matches!(extension, "mp4" | "mov" | "mkv" | "ts" | "avi"),
    }
}
