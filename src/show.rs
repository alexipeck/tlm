use crate::{config::Preferences, generic::Generic, model::*};
use tracing::{event, Level};

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Episode {
    pub episode_uid: Option<usize>,
    pub generic: Generic,
    pub show_uid: usize,
    pub show_title: String,
    pub show_season: usize,
    pub show_episode: Vec<usize>,
}

impl Episode {
    pub fn new(
        generic: Generic,
        show_uid: usize,
        show_title: String,
        show_season: usize,
        show_episode: Vec<usize>,
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

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn get_episode_string(&self) -> String {
        let episode = self.show_episode.clone();
        if episode.is_empty() {
            panic!("There was less than 1 episode in the thingo");
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
        event!(Level::DEBUG, "[generic_uid:'{:4}'][show_uid:'{:2}'][season:'{:2}'][episode:'{:2}'][full_path:'{}'][show_title:'{}']",
                self.generic.get_generic_uid(),
                self.show_uid,
                self.show_season,
                self.get_episode_string(),
                self.generic.get_full_path(),
                self.show_title,
        );
    }
}

#[derive(Clone, Debug)]
pub struct Season {
    pub number: usize,
    pub episodes: Vec<Episode>,
}

impl Season {
    pub fn new(number: usize) -> Season {
        Season {
            number,
            episodes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Show {
    pub show_uid: usize,
    pub show_title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, show_title: String) -> Show {
        Show {
            show_uid: uid,
            show_title,
            seasons: Vec::new(),
        }
    }

    pub fn insert_episode(&mut self, episode: Episode) {
        let season_number = episode.show_season;

        let mut found_season: bool = false;
        for season in &mut self.seasons {
            if season.number == season_number {
                found_season = true;
            }
        }

        if !found_season {
            self.seasons.push(Season::new(season_number))
        }

        for season in &mut self.seasons {
            if season.number != season_number {
                continue;
            }
            season.episodes.push(episode);
            break;
        }
    }

    pub fn print_show(&self, preferences: &Preferences) {
        if !preferences.print_shows && !preferences.show_output_whitelisted {
            return;
        }
        event!(
            Level::DEBUG,
            "[uid: {}][show_title: {}]",
            self.show_uid,
            self.show_title
        );
    }

    pub fn show_exists(
        show_title: String,
        working_shows: &[Show],
    ) -> Option<usize> {
        for s in working_shows {
            if s.show_title == show_title {
                return Some(s.show_uid);
            }
        }
        None
    }

    pub fn from_show_model(show_model: ShowModel) -> Show {
        let show_uid_temp: i32 = show_model.show_uid;
        let title_temp: String = show_model.show_title;

        let show = Show {
            show_uid: show_uid_temp as usize,
            show_title: title_temp,
            seasons: Vec::new(),
        };

        show
    }
}
