//!Module containing all structures used to represent a show
use crate::{
    config::Preferences,
    generic::{FileVersion, Generic},
    model::*,
};
use tracing::{debug, error};

///Structure contains all episode specific data as well as the underlying
///generic file data
#[derive(Clone, Debug)]
pub struct Episode {
    pub episode_uid: Option<usize>,
    pub generic: Generic,
    pub show_uid: i32,
    pub show_title: String,
    pub show_season: i32,
    pub show_episode: Vec<i32>,
}

impl Episode {
    pub fn new(
        generic: Generic,
        show_uid: i32,
        show_title: String,
        show_season: i32,
        show_episode: Vec<i32>,
    ) -> Self {
        Episode {
            episode_uid: None,
            generic,
            show_uid,
            show_title,
            show_season,
            show_episode,
        }
    }

    ///Convert the array of episodes to a string for printing
    fn get_episode_string(&self) -> String {
        let episode = self.show_episode.clone();
        if episode.is_empty() {
            error!("No episodes in show");
            panic!();
        } else {
            let mut episode_string = String::new();
            let mut first: bool = true;
            for episode in episode {
                if first {
                    episode_string.push_str(&format!("{}", episode));
                    first = false;
                } else {
                    episode_string += &format!("_{}", episode);
                }
            }
            episode_string
        }
    }

    pub fn print_episode(&self) {
        debug!("[generic_uid:'{:4}'][show_uid:'{:2}'][season:'{:2}'][episode:'{:2}'][full_path:'{}'][show_title:'{}']",
                self.generic.get_generic_uid(),
                self.show_uid,
                self.show_season,
                self.get_episode_string(),
                self.generic.get_master_full_path(),
                self.show_title,
        );
    }
}

///Structure to store a season containing the episodes of a show and the season number
#[derive(Clone, Debug)]
pub struct Season {
    pub number: i32,
    pub episodes: Vec<Episode>,
}

impl Season {
    pub fn new(number: i32) -> Season {
        Season {
            number,
            episodes: Vec::new(),
        }
    }

    //Returns true if successful
    pub fn insert_file_version(&mut self, file_version: &FileVersion) -> bool {
        for episode in self.episodes.iter_mut() {
            if episode.generic.generic_uid.is_none() {
                error!("This Episode's generic has no generic_uid, this shouldn't happen.");
                panic!();
            }
            if episode.generic.generic_uid.unwrap() == file_version.generic_uid {
                episode.generic.file_versions.push(file_version.to_owned());
                return true;
            }
        }
        false
    }
}

///Structure to represent a show containing seasons and the name of the show
#[derive(Clone, Debug)]
pub struct Show {
    pub show_uid: i32,
    pub show_title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: i32, show_title: String) -> Show {
        Show {
            show_uid: uid,
            show_title,
            seasons: Vec::new(),
        }
    }
    
    ///Add an episode to the show creating a season if none exists
    pub fn insert_episode(&mut self, episode: Episode) {
        let season_number = episode.show_season;

        let mut found_season: bool = false;
        for season in &mut self.seasons {
            if season.number == season_number {
                found_season = true;
            }
        }

        if !found_season {
            self.seasons.push(Season::new(season_number));
            debug!("Added {} season {}", self.show_title, season_number);
        }

        for season in &mut self.seasons {
            if season.number != season_number {
                continue;
            }
            season.episodes.push(episode);
            break;
        }
    }

    //Returns true if successful
    pub fn insert_file_version(&mut self, file_version: &FileVersion) -> bool {
        for season in self.seasons.iter_mut() {
            if season.insert_file_version(file_version) {
                return true;
            }
        }
        false
    }

    pub fn get_generic_from_uid(&self, generic_uid: i32) -> Option<Generic> {
        for season in &self.seasons {
            for episode in &season.episodes {
                //NOTE: Instead of panicking inside get_generic_uid(), it might be better to just pass over generic_uid's that are none,
                //      even though there shouldn't be generics that don't have a UID.
                //      This is fine for now
                if episode.generic.get_generic_uid() == generic_uid {
                    return Some(episode.generic.clone());
                }
            }
        }
        None
    }

    pub fn print_show(&self, preferences: &Preferences) {
        if !preferences.print_shows && !preferences.show_output_whitelisted {
            return;
        }
        debug!("[uid: {}][show_title: {}]", self.show_uid, self.show_title);
    }

    ///Create Show from ShowModel generated by database
    pub fn from_show_model(show_model: ShowModel) -> Show {
        let show_uid_temp: i32 = show_model.show_uid;
        let show_title: String = show_model.show_title;

        Show {
            show_uid: show_uid_temp,
            show_title,
            seasons: Vec::new(),
        }
    }
}
