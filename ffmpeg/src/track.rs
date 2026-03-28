#[derive(Debug, Clone)]
pub enum Track {
    Video {
        id: usize,
        codec: String,
        duration: std::time::Duration,
    },
    Audio {
        id: usize,
        codec: String,
        duration: std::time::Duration,
        language: Option<domain::language::LanguageCode>,
    },
    Subtitle {
        id: usize,
        language: Option<domain::language::LanguageCode>,
        external_id: Option<String>,
    },
}

impl Track {
    pub fn id(&self) -> &usize {
        match self {
            Track::Video { id, .. } => id,
            Track::Audio { id, .. } => id,
            Track::Subtitle { id, .. } => id,
        }
    }
}

pub async fn get_tracks() -> impl Iterator<Item = Track> {
    std::iter::empty()
}
