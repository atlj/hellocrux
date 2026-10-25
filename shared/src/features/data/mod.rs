mod contents;
mod downloads;
mod media;
mod preparing_list;
mod series;
mod tmdb;

use domain::{
    DownloadForm, TrackSelectionItem,
    series::{EditSeriesFileMappingForm, file_mapping_form_state},
};

use crate::{
    Model,
    features::data::{
        preparing_list::{handle_get_preparing, handle_send_track_selection},
        tmdb::search_tmdb_media,
    },
};

use contents::handle_get_contents;
use downloads::{handle_add_download, handle_get_downloads};
use media::handle_get_media;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum DataRequest {
    GetMedia,
    GetDownloads,
    GetPreparing,
    AddDownload(DownloadForm),
    GetContents(String),
    SetSeriesFileMapping(EditSeriesFileMappingForm<file_mapping_form_state::NeedsValidation>),
    SubmitTrackSelection(TrackSelectionItem),
    SearchTMDBMedia(String),
}

pub fn update_data(model: &mut Model, request: DataRequest) -> crate::Command {
    match request {
        DataRequest::SetSeriesFileMapping(form) => series::handle_file_mapping(model, form),
        DataRequest::GetContents(id) => handle_get_contents(model, id),
        DataRequest::GetMedia => handle_get_media(model),
        DataRequest::GetDownloads => handle_get_downloads(model),
        DataRequest::GetPreparing => handle_get_preparing(model),
        DataRequest::AddDownload(download_form) => handle_add_download(model, download_form),
        DataRequest::SubmitTrackSelection(track_selection_item) => {
            handle_send_track_selection(model, track_selection_item)
        }
        DataRequest::SearchTMDBMedia(query) => search_tmdb_media(model, query),
    }
}
