pub type PaginatedMultiSearchResultNonJoined = Paginated<ResultArrayItem>;
pub type PaginatedMultiSearchResult = Paginated<MultiSearchResult>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Paginated<T> {
    pub page: usize,
    pub total_pages: usize,
    pub total_results: usize,
    pub results: Vec<T>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MultiSearchResult {
    pub id: u32,
    pub title: String,
    pub poster_path: String,
    pub media_type: MediaType,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    TV,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "media_type", rename_all = "lowercase")]
pub enum ResultArrayItem {
    Movie(Movie),
    TV(TV),
    // We generally want to get rid of this
    Person(Person),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TV {
    id: u32,
    original_name: String,
    poster_path: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Movie {
    id: u32,
    original_title: String,
    poster_path: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Person {}

impl TryInto<MultiSearchResult> for ResultArrayItem {
    type Error = ();

    fn try_into(self) -> Result<MultiSearchResult, Self::Error> {
        match self {
            ResultArrayItem::Movie(Movie {
                id,
                original_title,
                poster_path: Some(poster_path),
            }) => Ok(MultiSearchResult {
                id,
                title: original_title,
                poster_path,
                media_type: MediaType::Movie,
            }),
            ResultArrayItem::TV(TV {
                id,
                original_name,
                poster_path: Some(poster_path),
            }) => Ok(MultiSearchResult {
                id,
                title: original_name,
                poster_path,
                media_type: MediaType::TV,
            }),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ErrorMessage {
    pub status_message: String,
}
