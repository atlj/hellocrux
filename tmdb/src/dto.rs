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
    pub original_title: String,
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
pub struct ErrorMessage {
    pub status_message: String,
}
