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

impl EpisodeIdentifier {
    pub fn find_earliest_available_episode(series: &SeriesContents) -> Option<EpisodeIdentifier> {
        let earliest_season_no = series.keys().min()?;
        let earliest_episode_no = series
            .get(earliest_season_no)
            .and_then(|season| season.keys().min())?;

        Some(EpisodeIdentifier {
            season_no: *earliest_season_no,
            episode_no: *earliest_episode_no,
        })
    }

    pub fn find_next_episode(&self, series: &SeriesContents) -> Option<EpisodeIdentifier> {
        // 1. See if there is a next episode in current season
        let current_season = series.get(&self.season_no)?;
        if let Some(next_episode_same_season) = current_season
            .keys()
            .filter(|episode_no| **episode_no > self.episode_no)
            .min()
        {
            return Some(self.with_episode_no(*next_episode_same_season));
        }

        // 2. Now see if there are any next seasons
        let (next_season, earliest_episode) = series
            .iter()
            .filter(|(season_no, _)| **season_no > self.season_no)
            .filter_map(|(season_no, episodes)| {
                let earliest_episode = episodes.keys().min()?;
                Some((*season_no, *earliest_episode))
            })
            .min_by(|a, b| a.0.cmp(&b.0))?;

        Some(EpisodeIdentifier {
            season_no: next_season,
            episode_no: earliest_episode,
        })
    }

    pub fn with_episode_no(&self, episode_no: u32) -> EpisodeIdentifier {
        EpisodeIdentifier {
            season_no: self.season_no,
            episode_no,
        }
    }

    pub fn with_season_no(&self, season_no: u32) -> EpisodeIdentifier {
        EpisodeIdentifier {
            season_no,
            episode_no: self.episode_no,
        }
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
    use std::{collections::HashMap, marker::PhantomData};

    use crate::{
        MediaPaths, SeasonContents, SeriesContents,
        series::{EditSeriesFileMappingForm, EpisodeIdentifier},
    };

    #[test]
    fn test_earliest_episode() {
        let empty_series: SeriesContents = HashMap::new();
        assert_eq!(
            EpisodeIdentifier::find_earliest_available_episode(&empty_series),
            None
        );

        let mut series_with_one_season: SeriesContents = HashMap::new();
        let mut season_with_two_episodes: SeasonContents = HashMap::new();
        season_with_two_episodes.insert(7, MediaPaths::default());
        season_with_two_episodes.insert(5, MediaPaths::default());
        series_with_one_season.insert(1, season_with_two_episodes);
        assert_eq!(
            EpisodeIdentifier::find_earliest_available_episode(&series_with_one_season),
            Some(EpisodeIdentifier {
                season_no: 1,
                episode_no: 5
            })
        );
    }

    #[test]
    fn test_next_episode() {
        // Existing test: empty series
        let empty_series: SeriesContents = HashMap::new();
        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 1,
        };
        assert_eq!(identifier.find_next_episode(&empty_series), None);

        // 1) Series has the season key, but the season is empty
        let mut series: SeriesContents = HashMap::new();
        series.insert(1, HashMap::new());
        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 1,
        };
        assert_eq!(identifier.find_next_episode(&series), None);

        // 2) Next episode exists in the same season (S1E1 -> S1E2)
        let mut series: SeriesContents = HashMap::new();
        let mut s1: SeasonContents = HashMap::new();
        s1.insert(1, MediaPaths::default());
        s1.insert(2, MediaPaths::default());
        series.insert(1, s1);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 1,
        };
        assert_eq!(
            identifier.find_next_episode(&series),
            Some(EpisodeIdentifier {
                season_no: 1,
                episode_no: 2
            })
        );

        // 3) Gap in episode numbers: S1E1, S1E3 only (S1E1 should go to S1E3 if you pick "next available")
        // If your intended behavior is strictly +1, change expected to None.
        let mut series: SeriesContents = HashMap::new();
        let mut s1: SeasonContents = HashMap::new();
        s1.insert(1, MediaPaths::default());
        s1.insert(3, MediaPaths::default());
        series.insert(1, s1);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 1,
        };
        assert_eq!(
            identifier.find_next_episode(&series),
            Some(EpisodeIdentifier {
                season_no: 1,
                episode_no: 3
            })
        );

        // 4) End of season -> next season first episode: S1E2 -> S2E1
        let mut series: SeriesContents = HashMap::new();
        let mut s1: SeasonContents = HashMap::new();
        s1.insert(1, MediaPaths::default());
        s1.insert(2, MediaPaths::default());
        let mut s2: SeasonContents = HashMap::new();
        s2.insert(1, MediaPaths::default());
        series.insert(1, s1);
        series.insert(2, s2);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 2,
        };
        assert_eq!(
            identifier.find_next_episode(&series),
            Some(EpisodeIdentifier {
                season_no: 2,
                episode_no: 1
            })
        );

        // 5) End of season but next season missing -> None
        let mut series: SeriesContents = HashMap::new();
        let mut s1: SeasonContents = HashMap::new();
        s1.insert(1, MediaPaths::default());
        s1.insert(2, MediaPaths::default());
        series.insert(1, s1);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 2,
        };
        assert_eq!(identifier.find_next_episode(&series), None);

        // 6) Current season missing entirely -> None (even if later seasons exist)
        let mut series: SeriesContents = HashMap::new();
        let mut s2: SeasonContents = HashMap::new();
        s2.insert(1, MediaPaths::default());
        series.insert(2, s2);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 1,
        };
        assert_eq!(identifier.find_next_episode(&series), None);

        // 7) Current episode not present but later episode exists (S1E2 missing, has E1 and E3)
        // Depending on implementation: could return E3 (next available) or None (if requires current to exist).
        // Adjust expected accordingly.
        let mut series: SeriesContents = HashMap::new();
        let mut s1: SeasonContents = HashMap::new();
        s1.insert(1, MediaPaths::default());
        s1.insert(3, MediaPaths::default());
        series.insert(1, s1);

        let identifier = EpisodeIdentifier {
            season_no: 1,
            episode_no: 2,
        };
        assert_eq!(
            identifier.find_next_episode(&series),
            Some(EpisodeIdentifier {
                season_no: 1,
                episode_no: 3
            })
        );
    }

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
