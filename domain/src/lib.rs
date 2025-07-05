use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Media {
    pub id: String,
    pub metadata: MediaMetaData,
    pub content: MediaContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaContent {
    Movie(String),
    Series(HashMap<u32, HashMap<u32, String>>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaMetaData {
    pub thumbnail: String,
    pub title: String,
}
