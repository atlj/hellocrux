#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum QueryState<T>
where
    T: serde::Serialize,
{
    Loading { data: Option<T> },
    Success { data: T },
    Error { message: String },
}

impl<T> std::default::Default for QueryState<T>
where
    T: serde::Serialize,
{
    fn default() -> Self {
        Self::Loading { data: None }
    }
}

impl<T> QueryState<T>
where
    T: serde::Serialize,
{
    pub fn get_data(&self) -> Option<&T> {
        match &self {
            QueryState::Loading { data } => data.as_ref(),
            QueryState::Success { data } => Some(data),
            QueryState::Error { .. } => None,
        }
    }

    pub fn as_ref(&self) -> QueryState<&T> {
        use QueryState as E;
        match self {
            E::Loading { data } => E::Loading {
                data: data.as_ref(),
            },
            E::Success { data } => E::Success { data },
            E::Error { message } => E::Error {
                message: message.clone(),
            },
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

macro_rules! query_state_type {
    ($name:ident, $data:ty) => {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        pub enum $name {
            Loading { data: Option<$data> },
            Success { data: $data },
            Error { message: String },
        }

        impl Default for $name {
            fn default() -> Self {
                Self::Loading { data: None }
            }
        }

        impl From<QueryState<$data>> for $name {
            fn from(value: QueryState<$data>) -> Self {
                match value {
                    QueryState::Loading { data } => $name::Loading { data },
                    QueryState::Success { data } => $name::Success { data },
                    QueryState::Error { message } => $name::Error { message },
                }
            }
        }
    };
}

pub mod view_model_queries {
    use std::collections::HashMap;

    use domain::{Media, language::LanguageCode, series::EpisodeIdentifier};

    use crate::features::query::QueryState;

    pub type MediaItemsContent = HashMap<String, Media>;

    query_state_type!(ActionState, ());
    query_state_type!(MediaItems, MediaItemsContent);
    query_state_type!(SubtitleSearchState, SubtitleSearchResults);

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub enum SubtitleSearchResults {
        Movie {
            media_id: String,
            language: LanguageCode,
            options: Vec<SubtitleSearchResult>,
        },

        Series {
            media_id: String,
            language: LanguageCode,
            options: HashMap<EpisodeIdentifier, Vec<SubtitleSearchResult>>,
        },
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub struct SubtitleSearchResult {
        pub id: usize,
        pub title: String,
        pub download_count: usize,
    }

    impl From<domain::subtitles::SubtitleDownloadOption<usize>> for SubtitleSearchResult {
        fn from(value: domain::subtitles::SubtitleDownloadOption<usize>) -> Self {
            Self {
                id: value.id,
                title: value.title,
                download_count: value.download_count,
            }
        }
    }
}
