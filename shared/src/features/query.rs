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

pub mod view_model_queries {
    // TODO reduce repetition
    use std::collections::HashMap;

    use domain::Media;

    use crate::features::query::QueryState;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
    pub enum ConnectionState {
        #[default]
        Loading,
        Success,
        Error {
            message: String,
        },
    }

    impl From<QueryState<()>> for ConnectionState {
        fn from(value: QueryState<()>) -> Self {
            match value {
                QueryState::Loading { .. } => Self::Loading,
                QueryState::Success { .. } => Self::Success,
                QueryState::Error { message } => Self::Error { message },
            }
        }
    }

    pub type MediaItemsContent = HashMap<String, Media>;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    pub enum MediaItems {
        Loading { data: Option<MediaItemsContent> },
        Success { data: MediaItemsContent },
        Error { message: String },
    }

    impl Default for MediaItems {
        fn default() -> Self {
            Self::Loading { data: None }
        }
    }

    impl From<QueryState<MediaItemsContent>> for MediaItems {
        fn from(value: QueryState<MediaItemsContent>) -> Self {
            match value {
                QueryState::Loading { data } => MediaItems::Loading { data },
                QueryState::Success { data } => MediaItems::Success { data },
                QueryState::Error { message } => MediaItems::Error { message },
            }
        }
    }
}
