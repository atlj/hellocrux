use std::{collections::HashMap, path::Path};

use domain::series::EpisodeIdentifier;

use crate::{
    Model, PartialModel,
    capabilities::{
        http,
        navigation::{self, Screen},
    },
    features::utils::update_model,
};

pub fn handle_get_contents(model: &Model, id: String) -> crate::Command {
    let base_url = model.base_url.clone();

    crate::Command::new(|ctx| async move {
        // TODO make this url common between enum vals
        let url = {
            let mut url = if let Some(url) = base_url {
                url
            } else {
                return navigation::push(Screen::ServerAddressEntry)
                    .into_future(ctx)
                    .await;
            };

            url.set_path("download/torrent-contents");
            url.set_query(Some(format!("id={id}").as_ref()));
            url
        };

        match http::get(url).into_future(ctx.clone()).await {
            http::HttpOutput::Success { data, .. } => {
                let files = data
                    .and_then(|data| serde_json::from_str::<Vec<String>>(&data).ok())
                    .map(|files| {
                        let len = files.len();
                        let map =
                            files
                                .into_iter()
                                .fold(HashMap::with_capacity(len), |mut map, file| {
                                    if let Some(extension) = (file.as_ref() as &Path).extension() {
                                        // TODO make this a generic check
                                        if extension == "srt" || extension == "nfo" {
                                            return map;
                                        }
                                    }

                                    let identifier = detect_episode_identifier(&file).unwrap_or(
                                        EpisodeIdentifier {
                                            season_no: 1,
                                            episode_no: 1,
                                        },
                                    );
                                    map.insert(file, identifier);
                                    map
                                });
                        (id, map)
                    });

                update_model(
                    &ctx,
                    PartialModel {
                        torrent_contents: Some(files),
                        ..Default::default()
                    },
                );
            }
            http::HttpOutput::Error => {
                // TODO: add logging
            }
        }
    })
}

pub fn detect_episode_identifier(path: &str) -> Option<EpisodeIdentifier> {
    let re = regex::Regex::new(r".*S([0-9]+)E([0-9]+).*").expect("Invalid regex supplied");
    let first_capture = re.captures_iter(path).next()?;
    let (_, [season_no_str, episode_no_str]) = first_capture.extract();

    Some(EpisodeIdentifier {
        season_no: season_no_str.parse().ok()?,
        episode_no: episode_no_str.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use domain::series::EpisodeIdentifier;

    use super::detect_episode_identifier;

    #[test]
    fn test_detect_episode_identifier() {
        assert_eq!(
            detect_episode_identifier("my-series.S02E01.1080p.x265-HEYYY.mkv").unwrap(),
            EpisodeIdentifier {
                season_no: 2,
                episode_no: 1
            }
        );

        assert!(detect_episode_identifier("my-series.S02E.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.S02.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.02.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.E2S7.1080p.x265-HEYYY.mkv").is_none());
        assert!(detect_episode_identifier("my-series.SE.1080p.x265-HEYYY.mkv").is_none());
    }
}
