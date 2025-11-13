use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EpisodeIdentifier {
    pub season_no: u32,
    pub episode_no: u32,
}

// TODO unit tests
impl EpisodeIdentifier {
    pub fn find_earliest_available_episode(
        series: &HashMap<u32, HashMap<u32, String>>,
    ) -> EpisodeIdentifier {
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

    pub fn find_next_episode(
        &self,
        series: &HashMap<u32, HashMap<u32, String>>,
    ) -> Option<EpisodeIdentifier> {
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
