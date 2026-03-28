mod contents;
mod downloads;
mod media;
mod series;

use domain::{
    DownloadForm,
    series::{EditSeriesFileMappingForm, file_mapping_form_state},
};

use crate::Model;

use contents::handle_get_contents;
use downloads::{handle_add_download, handle_get_downloads};
use media::handle_get_media;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum DataRequest {
    GetMedia,
    GetDownloads,
    AddDownload(DownloadForm),
    GetContents(String),
    SetSeriesFileMapping(EditSeriesFileMappingForm<file_mapping_form_state::NeedsValidation>),
}

pub fn update_data(model: &mut Model, request: DataRequest) -> crate::Command {
    match request {
        DataRequest::SetSeriesFileMapping(form) => series::handle_file_mapping(model, form),
        DataRequest::GetContents(id) => handle_get_contents(model, id),
        DataRequest::GetMedia => handle_get_media(model),
        DataRequest::GetDownloads => handle_get_downloads(model),
        DataRequest::AddDownload(download_form) => handle_add_download(model, download_form),
    }
}
