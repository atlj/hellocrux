use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use crate::SeriesContents;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct EpisodeIdentifier {
    pub season_no: u32,
    pub episode_no: u32,
}

// TODO unit tests
impl EpisodeIdentifier {
    pub fn find_earliest_available_episode(series: &SeriesContents) -> EpisodeIdentifier {
        let earliest_season_no = series
            .keys()
            .min()
            .expect("We should have at least one season");
        let earliest_episode_no = series
            .get(earliest_season_no)
            .and_then(|season| season.keys().min())
            .expect("The season must have at least one episode");

        EpisodeIdentifier {
            season_no: *earliest_season_no,
            episode_no: *earliest_episode_no,
        }
    }

    // TODO rewrite
    pub fn find_next_episode(&self, series: &SeriesContents) -> Option<EpisodeIdentifier> {
        // 1. Next episode is available
        let current_season_episodes = series.get(&self.season_no).unwrap();
        let next_episode_in_current_season = current_season_episodes
            .keys()
            .filter(|episode| **episode > self.episode_no)
            .min();
        if let Some(next_episode_in_current_season) = next_episode_in_current_season {
            return Some(EpisodeIdentifier {
                season_no: self.season_no,
                episode_no: *next_episode_in_current_season,
            });
        }

        // 2. Next season is available
        let next_season_no = series
            .keys()
            .filter(|season| **season > self.season_no)
            .min()?; // <- Tricky None return if there is no next season

        let earliest_episode_in_next_season = series
            .get(next_season_no)
            .and_then(|season| season.keys().min())
            .expect("The season must have at least one episode");

        Some(EpisodeIdentifier {
            season_no: *next_season_no,
            episode_no: *earliest_episode_in_next_season,
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct EditSeriesFileMappingForm<T> {
    pub id: Box<str>,
    pub file_mapping: SeriesFileMapping,

    #[serde(skip)]
    pub phantom: PhantomData<T>,
}

impl EditSeriesFileMappingForm<file_mapping_form_state::NeedsValidation> {
    pub fn validate(
        self,
        allowed_files: &[String],
    ) -> Option<EditSeriesFileMappingForm<file_mapping_form_state::Valid>> {
        let has_unknown_files = {
            self.file_mapping
                .keys()
                .any(|file| !allowed_files.contains(file))
        };
        if has_unknown_files {
            return None;
        }

        let has_duplicates = {
            let entries_hash_set: HashSet<&_, std::hash::RandomState> =
                HashSet::from_iter(self.file_mapping.values());
            self.file_mapping.len() > entries_hash_set.len()
        };
        if has_duplicates {
            return None;
        }

        Some(EditSeriesFileMappingForm {
            id: self.id,
            file_mapping: self.file_mapping,
            phantom: PhantomData,
        })
    }
}

pub mod file_mapping_form_state {
    #[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
    pub struct Valid {}
    #[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
    pub struct NeedsValidation {}
}

/// Key is file path
pub type SeriesFileMapping = HashMap<String, EpisodeIdentifier>;

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::series::{EditSeriesFileMappingForm, EpisodeIdentifier};

    #[test]
    fn validate_form_mapping_form() {
        assert!(
            EditSeriesFileMappingForm {
                id: "hey".into(),
                file_mapping: [(
                    "hello/worldS1E1.mov".to_string(),
                    EpisodeIdentifier {
                        season_no: 1,
                        episode_no: 1
                    }
                )]
                .into(),
                phantom: PhantomData
            }
            .validate(&["hello/worldS1E1.mov".to_string()])
            .is_some()
        );

        assert!(
            EditSeriesFileMappingForm {
                id: "hey".into(),
                file_mapping: [(
                    "some/malicious/path".to_string(),
                    EpisodeIdentifier {
                        season_no: 1,
                        episode_no: 1
                    }
                )]
                .into(),
                phantom: PhantomData
            }
            .validate(&["hello/worldS1E1.mov".to_string()])
            .is_none()
        );

        assert!(
            EditSeriesFileMappingForm {
                id: "hey".into(),
                file_mapping: [
                    (
                        "hello/worldS1E1.mov".to_string(),
                        EpisodeIdentifier {
                            season_no: 1,
                            episode_no: 1
                        },
                    ),
                    (
                        "hello/worldS1E2.mov".to_string(),
                        EpisodeIdentifier {
                            season_no: 1,
                            episode_no: 2
                        }
                    )
                ]
                .into(),
                phantom: PhantomData
            }
            .validate(&[
                "hello/worldS1E1.mov".to_string(),
                "hello/worldS1E2.mov".to_string()
            ])
            .is_some()
        );

        assert!(
            EditSeriesFileMappingForm {
                id: "hey".into(),
                file_mapping: [
                    (
                        "hello/worldS1E1.mov".to_string(),
                        EpisodeIdentifier {
                            season_no: 1,
                            episode_no: 1
                        },
                    ),
                    (
                        "hello/worldS1E2.mov".to_string(),
                        EpisodeIdentifier {
                            season_no: 1,
                            episode_no: 1
                        }
                    )
                ]
                .into(),
                phantom: PhantomData
            }
            .validate(&[
                "hello/worldS1E1.mov".to_string(),
                "hello/worldS1E2.mov".to_string()
            ])
            .is_none()
        );
    }
}
