//!A struct for managing all types of media that are stored in ram as well as
//!Functionality to import files. This is mostly used in the scheduler
use crate::{
    config::{Config, Preferences},
    database::*,
    designation::Designation,
    generic::Generic,
    model::{NewEpisode, NewGeneric},
    show::{Episode, Show},
};
use diesel::pg::PgConnection;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, path::PathBuf};
use tracing::{event, Level};
use walkdir::{DirEntry, WalkDir};

///Struct to hold all root directories containing media
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct TrackedDirectories {
    pub root_directories: Vec<String>,
    pub cache_directories: Vec<String>,
}

impl TrackedDirectories {
    pub fn new() -> TrackedDirectories {
        TrackedDirectories {
            root_directories: Vec::new(),
            cache_directories: Vec::new(),
        }
    }
}

///This enum represents the list of reasons that a file was not imported.
///This is used to create a log of files that weren't imported so
///that the user can determine the reason that some of their media
///was not imported if they expected it to be
#[derive(Debug, Clone)]
pub enum Reason {
    PathContainsIgnoredPath,
    ExtensionMissing,
    ExtensionDisallowed,
}

///Convert a vector of reasons to a vector of strings. Used to convert all
///reasons for a particular file
fn reasons_to_string(reasons: Vec<Reason>) -> String {
    reasons
        .iter()
        .map(|reason| format!("[{}]", reason.to_string()))
        .collect()
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted: &str = match self {
            Self::PathContainsIgnoredPath => "PathContainsIgnoredPath",
            Self::ExtensionMissing => "ExtensionMissing",
            Self::ExtensionDisallowed => "ExtensionDisallowed",
        };

        write!(f, "{}", formatted)
    }
}

///Contains all media data that is stored in ram as well as
///a list of rejected files
pub struct FileManager {
    ///copy of the one in the scheduler
    pub config: Config,
    pub generic_files: Vec<Generic>,
    pub shows: Vec<Show>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub new_files_queue: Vec<PathBuf>,

    pub rejected_files: Vec<(PathBuf, Vec<Reason>)>,
    pub rejected_files_hashset: HashSet<PathBuf>,
}

impl FileManager {
    pub fn new(config: &Config) -> FileManager {
        let mut file_manager = FileManager {
            config: config.clone(),
            shows: get_all_shows(),
            generic_files: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),

            rejected_files: Vec::new(),
            rejected_files_hashset: HashSet::new(),
        };

        //add generic_files and generics from their respective episodes to the existing_files_hashset
        file_manager.generic_files = get_all_generics();

        file_manager.add_existing_files_to_hashset();
        file_manager.add_show_episodes_to_hashset();

        file_manager
    }

    ///Takes all loaded episodes and add them to the hashset of
    ///existing files to ensure that files don't get imported twice
    fn add_show_episodes_to_hashset(&mut self) {
        let mut generics: Vec<Generic> = Vec::new();
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    generics.push(episode.generic.clone());
                }
            }
        }

        self.add_all_filenames_to_hashset_from_generics(&generics);
    }

    ///Adds all generics that exist in the file manager to the hashset to ensure that
    ///files can't be imported twice
    fn add_existing_files_to_hashset(&mut self) {
        for generic in &self.generic_files {
            self.existing_files_hashset
                .insert(generic.full_path.clone());
        }
    }

    ///Add a collection of generics to the hashset of generics to ensure
    ///that files can't be imported twice
    fn add_all_filenames_to_hashset_from_generics(&mut self, generics: &[Generic]) {
        for generic in generics {
            self.existing_files_hashset
                .insert(generic.full_path.clone());
        }
    }

    pub fn print_number_of_generics(&self) {
        event!(
            Level::INFO,
            "Number of generics loaded in memory: {}",
            self.generic_files.len()
        );
    }

    pub fn print_number_of_shows(&self) {
        event!(
            Level::INFO,
            "Number of shows loaded in memory: {}",
            self.shows.len()
        );
    }

    pub fn print_number_of_episodes(&self) {
        let mut episode_counter = 0;
        for show in &self.shows {
            for season in &show.seasons {
                episode_counter += season.episodes.len();
            }
        }

        event!(
            Level::INFO,
            "Number of episodes loaded in memory: {}",
            episode_counter
        );
    }

    ///Processes all files in the new files queue and converts them to
    ///episodes and generics based on pattern matching. So far only accepts
    ///The filename pattern SxxExx where Sxx is the season number and Exx
    ///is the episode number
    pub fn process_new_files(&mut self, preferences: &Preferences) {
        let connection = establish_connection();
        let mut new_episodes = Vec::new();
        let mut new_generics = Vec::new();

        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"S[0-9]*E[0-9\-]*").unwrap();
        }

        //Create Content and NewContent that will be added to the database in a batch
        let mut temp_generics: Vec<Generic> = self
            .new_files_queue
            .par_iter()
            .map(|current| {
                let mut generic = Generic::new(current);

                //TODO: Why yes this is slower, no I don't care about 100ms right now
                match REGEX.find(&generic.get_filename()) {
                    None => {}
                    Some(_) => generic.designation = Designation::Episode,
                }

                generic
            })
            .collect();
        self.new_files_queue.clear();
        for generic in &temp_generics {
            new_generics.push(NewGeneric::new(
                String::from(generic.full_path.to_str().unwrap()),
                generic.designation as i32,
                generic.profile,
            ));
        }

        //Insert the generic and then update the uid's for the full Generic structure
        let generics = create_generics(&connection, new_generics);
        for i in 0..generics.len() {
            temp_generics[i].generic_uid = Some(generics[i].generic_uid as usize);
        }

        //Build all the NewEpisodes so we can do a batch insert that is faster than doing one at a time in a loop
        for generic in &mut temp_generics {
            let episode_string: String;
            match REGEX.find(&generic.get_filename()) {
                None => continue,
                Some(val) => {
                    //Removes first character
                    let mut chars = val.as_str().chars();
                    chars.next();

                    episode_string = String::from(chars.as_str());
                }
            }

            let mut season_episode_iter = episode_string.split('E');
            let season_temp = season_episode_iter
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let mut episodes: Vec<usize> = Vec::new();
            for episode in season_episode_iter.next().unwrap().split('-') {
                episodes.push(episode.parse::<usize>().unwrap());
            }

            generic.designation = Designation::Episode;
            let show_title = generic
                .full_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let show_uid = self.ensure_show_exists(show_title.clone(), &connection, preferences);
            let season_number = season_temp;
            let episode_number = episodes[0];

            let new_episode = NewEpisode::new(
                generic.get_generic_uid(),
                show_uid,
                "".to_string(), //episode_title
                season_number,
                episode_number,
            );
            new_episodes.push(new_episode);
        }

        self.add_all_filenames_to_hashset_from_generics(&temp_generics);

        let mut temp_generics_only_episodes: Vec<Generic> = Vec::new();
        let mut temp_generics_only_generics: Vec<Generic> = Vec::new();
        for generic in &temp_generics {
            match generic.designation {
                Designation::Generic => temp_generics_only_generics.push(generic.clone()),
                Designation::Episode => temp_generics_only_episodes.push(generic.clone()),
                _ => {}
            }
        }

        self.generic_files.append(&mut temp_generics_only_generics);

        let episode_models = create_episodes(&connection, new_episodes);
        let mut episodes: Vec<Episode> = Vec::new();
        for episode_model in episode_models {
            for generic in &temp_generics_only_episodes {
                if generic.get_generic_uid() == episode_model.generic_uid as usize {
                    let episode = Episode::new(
                        generic.clone(),
                        episode_model.show_uid as usize,
                        "".to_string(),
                        episode_model.season_number as usize,
                        vec![episode_model.episode_number as usize],
                    ); //temporary first episode_number
                    episodes.push(episode);
                    break;
                }
            }
        }

        self.insert_episodes(episodes);
    }

    ///returns none when a file is rejected because is accepted, or already exists in the existing_files_hashset
    fn accept_or_reject_file(
        &mut self,
        dir_entry: DirEntry,
        read_only: bool,
    ) -> Option<Vec<Reason>> {
        let path = dir_entry.path();
        let mut reason: Vec<Reason> = Vec::new();
        let mut allowed: bool = true;

        //rejects if the path contains any element of an ignored path
        for ignored_path in &self.config.ignored_paths {
            if path
                .to_str()
                .unwrap()
                .to_lowercase()
                .contains(&ignored_path.to_lowercase())
            {
                reason.push(Reason::PathContainsIgnoredPath);
                allowed = false;
            }
        }

        //rejects if the path doesn't have an extension
        if dir_entry.path().extension().is_none() {
            reason.push(Reason::ExtensionMissing);
            allowed = false;
        } else {
            //rejects if the file doesn't have an allowed extension
            if !self
                .config
                .allowed_extensions
                .contains(&path.extension().unwrap().to_str().unwrap().to_lowercase())
            {
                reason.push(Reason::ExtensionDisallowed);
                allowed = false;
            }
        }

        let entry_string = dir_entry.into_path();

        //rejects if the file exists in the existing_files_hashset
        if self.existing_files_hashset.contains(&entry_string) {
            return None;
        }

        if read_only {
            return Some(reason);
        }

        if allowed {
            self.existing_files_hashset.insert(entry_string.clone());
            self.new_files_queue.push(entry_string);
        } else if !self.rejected_files_hashset.contains(&entry_string) {
            self.rejected_files_hashset.insert(entry_string.clone());
            self.rejected_files.push((entry_string, reason));
        }

        None
    }

    ///Import all files in the list of tracked root directories
    ///into a queue for later processing. Uses a Hash set to
    ///guarantee no duplicates in O(1) time
    pub fn import_files(&mut self) {
        //import all files in tracked root directories
        for directory in &self.config.tracked_directories.root_directories.clone() {
            let entries = WalkDir::new(directory).into_iter().filter_map(|e| e.ok());
            for entry in entries {
                //rejects if the path isn't a file
                if !entry.path().is_file() {
                    continue;
                }

                let reason = self.accept_or_reject_file(entry.clone(), false);

                if reason.is_some() {
                    event!(
                        Level::INFO,
                        "Path: '{}' disallowed because {}",
                        String::from(entry.into_path().to_str().unwrap()),
                        reasons_to_string(reason.unwrap())
                    );
                }
            }
        }
    }

    pub fn print_episodes(&self, preferences: &Preferences) {
        if !preferences.print_episode && !preferences.episode_output_whitelisted {
            return;
        }
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    episode.print_episode();
                }
            }
        }
    }

    //TODO: This shouldn't call another version in generic
    //it should just use the logic of the other function here
    pub fn print_generics(&self, preferences: &Preferences) {
        Generic::print_generics(&self.generic_files, preferences);
    }

    ///Insert a vector of episodes into an existing show
    pub fn insert_episodes(&mut self, episodes: Vec<Episode>) {
        //find the associated show
        //insert episode into that show
        for episode in episodes {
            let show_uid = episode.show_uid;
            for show in &mut self.shows {
                if show.show_uid == show_uid {
                    show.insert_episode(episode);
                    break;
                }
            }
        }
    }

    ///Check if a show exists in ram
    fn show_exists(&self, show_title: String) -> Option<usize> {
        for s in &self.shows {
            if s.show_title == show_title {
                return Some(s.show_uid);
            }
        }
        None
    }

    ///Make sure a show exists by checking for it in ram and inserting it into
    ///the database if it doesn't exist yet
    fn ensure_show_exists(
        &mut self,
        show_title: String,
        connection: &PgConnection,
        preferences: &Preferences,
    ) -> usize {
        let show_uid = self.show_exists(show_title.clone());
        match show_uid {
            Some(uid) => uid,
            None => {
                if preferences.print_shows || preferences.show_output_whitelisted {
                    event!(Level::DEBUG, "Adding a new show: {}", show_title);
                }

                let show_model = create_show(connection, show_title.clone());

                let show_uid = show_model.show_uid as usize;
                let new_show = Show {
                    show_uid,
                    show_title,
                    seasons: Vec::new(),
                };
                self.shows.push(new_show);

                show_uid
            }
        }
    }

    pub fn print_shows(&self, preferences: &Preferences) {
        if !preferences.print_shows {
            return;
        }
        for show in &self.shows {
            show.print_show(preferences);
            for season in &show.seasons {
                event!(
                    Level::DEBUG,
                    "S{:02} has {} episodes",
                    season.number,
                    season.episodes.len()
                );
            }
        }
    }

    pub fn print_rejected_files(&self) {
        for (pathbuf, reason) in &self.rejected_files {
            event!(
                Level::INFO,
                "Path: '{}' disallowed because {}",
                String::from(pathbuf.to_str().unwrap()),
                reasons_to_string(reason.clone())
            );
        }
    }
}
