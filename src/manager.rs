use crate::model::{NewEpisode, NewGeneric};

use crate::{
    config::Config,
    database::*,
    designation::Designation,
    generic::Generic,
    print::{print, From, Verbosity},
    tv::{Episode, Show},
    utility::Utility,
};
use diesel::pg::PgConnection;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

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

pub struct FileManager {
    pub tracked_directories: TrackedDirectories,
    pub generic_files: Vec<Generic>,
    pub shows: Vec<Show>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub new_files_queue: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(config: &Config, utility: Utility) -> FileManager {
        let mut utility = utility.clone_add_location("new(FileManager)");

        let mut file_manager = FileManager {
            tracked_directories: TrackedDirectories::new(),
            shows: get_all_shows(utility.clone()),
            generic_files: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
        };

        //add generic_files and generics from their respective episodes to the existing_files_hashset
        file_manager.generic_files = get_all_generics(utility.clone());

        file_manager.add_existing_files_to_hashset(utility.clone());
        file_manager.add_show_episodes_to_hashset(utility.clone());
        file_manager.tracked_directories = config.tracked_directories.clone();

        utility.print_function_timer();
        file_manager
    }

    pub fn add_show_episodes_to_hashset(&mut self, utility: Utility) {
        let mut generics: Vec<Generic> = Vec::new();
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    generics.push(episode.generic.clone());
                }
            }
        }

        self.add_all_filenames_to_hashset_from_generics(&generics, utility);
    }

    fn add_existing_files_to_hashset(&mut self, utility: Utility) {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        for generic in &self.generic_files {
            self.existing_files_hashset
                .insert(generic.full_path.clone());
        }

        utility.print_function_timer();
    }

    pub fn add_all_filenames_to_hashset_from_generics(
        &mut self,
        generics: &[Generic],
        utility: Utility,
    ) {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        for generic in generics {
            self.existing_files_hashset
                .insert(generic.full_path.clone());
        }

        utility.print_function_timer();
    }

    pub fn print_number_of_generic(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_number_of_generic(FileManager)");

        print(
            Verbosity::INFO,
            From::Manager,
            format!(
                "Number of generic loaded in memory: {}",
                self.generic_files.len()
            ),
            false,
            utility,
        );
    }

    pub fn print_number_of_shows(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_number_of_shows(FileManager)");

        print(
            Verbosity::INFO,
            From::Manager,
            format!("Number of shows loaded in memory: {}", self.shows.len()),
            false,
            utility,
        );
    }

    pub fn process_new_files(&mut self, progress_bar: &ProgressBar, utility: Utility) {
        let mut utility = utility.clone_add_location("process_new_files(FileManager)");
        let connection = establish_connection();
        let mut new_episodes = Vec::new();
        let mut new_generics = Vec::new();

        //Will just be appended to working content at the end
        let mut temp_generics = Vec::new();
        progress_bar.set_length(self.new_files_queue.len() as u64);
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"S[0-9]*E[0-9\-]*").unwrap();
        }

        //Create Content and NewContent that will be added to the database in a batch
        while !self.new_files_queue.is_empty() {
            let current = self.new_files_queue.pop();
            if let Some(current) = current {
                /*progress_bar.set_message(format!(
                    "processing file: {}",
                    current.file_name().unwrap().to_str().unwrap()
                ));*/

                let mut generic = Generic::new(&current, utility.clone());
                progress_bar.inc(1);

                //TODO: Why yes this is slower, no I don't care abou 100ms right now
                match REGEX.find(&generic.get_filename()) {
                    None => {}
                    Some(_) => generic.designation = Designation::Episode,
                }

                if generic.profile.is_some() {
                    let profile = generic.profile.unwrap();

                    new_generics.push(NewGeneric {
                        full_path: String::from(generic.full_path.to_str().unwrap()),
                        designation: generic.designation as i32,
                        width: Some(profile.width as i32),
                        height: Some(profile.height as i32),
                        framerate: Some(profile.framerate),
                        length_time: Some(profile.length_time),
                    });
                } else {
                    new_generics.push(NewGeneric {
                        full_path: String::from(generic.full_path.to_str().unwrap()),
                        designation: generic.designation as i32,
                        width: None,
                        height: None,
                        framerate: None,
                        length_time: None,
                    });
                }

                temp_generics.push(generic);
            }
        }

        //Insert the generic and then update the uid's for the full Generic structure
        let generics = create_generics(&connection, new_generics);
        for i in 0..generics.len() {
            temp_generics[i].generic_uid = Some(generics[i].generic_uid as usize);
        }

        //Build all the NewEpisodes so we can do a batch insert that is faster than doing one at a time in a loop
        for generic in &mut temp_generics {
            progress_bar.set_message(format!(
                "creating episode: {}",
                generic.full_path.file_name().unwrap().to_str().unwrap()
            ));

            let episode_string: String;
            match REGEX.find(&generic.get_filename()) {
                None => continue,
                Some(val) => episode_string = String::from(rem_first_char(val.as_str())),
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

            let show_uid =
                self.ensure_show_exists(show_title.clone(), utility.clone(), &connection);
            let season_number = season_temp;
            let episode_number = episodes[0];

            let new_episode = NewEpisode::new(
                generic.get_generic_uid(utility.clone()),
                show_uid,
                "".to_string(), //episode_title
                season_number,
                episode_number,
            );
            new_episodes.push(new_episode);
        }

        self.generic_files.append(&mut temp_generics);

        //episodes isn't being used yet but this does insert into the database
        let episode_models = create_episodes(&connection, new_episodes);
        let mut episodes: Vec<Episode> = Vec::new();

        for episode_model in episode_models {
            for generic in &self.generic_files {
                if generic.get_generic_uid(utility.clone()) == episode_model.generic_uid as usize {
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

        self.insert_episodes(episodes, utility.clone());

        progress_bar.finish();
        utility.print_function_timer();

        fn rem_first_char(value: &str) -> &str {
            let mut chars = value.chars();
            chars.next();
            chars.as_str()
        }
    }

    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(
        &mut self,
        allowed_extensions: &[String],
        ignored_paths: &[String],
        progress_bar: &ProgressBar,
        utility: Utility,
    ) {
        let mut utility = utility.clone_add_location("import_files(FileManager)");

        //Return true if string contains any substring from Vector
        fn str_contains_strs(input_str: &str, substrings: &[String]) -> bool {
            for substring in substrings {
                if String::from(input_str).contains(&substring.to_lowercase()) {
                    return true;
                }
            }
            false
        }

        //import all files in tracked root directories
        for directory in &self.tracked_directories.root_directories {
            for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
                if str_contains_strs(
                    &entry.path().to_str().unwrap().to_lowercase(),
                    ignored_paths,
                ) {
                    break;
                }
                if entry.path().is_file() {
                    let temp_string = entry.path().extension().unwrap().to_str().unwrap();
                    if allowed_extensions.contains(&temp_string.to_lowercase()) {
                        let entry_string = entry.into_path();
                        progress_bar.set_message(format!(
                            "importing files: {}",
                            entry_string.file_name().unwrap().to_str().unwrap()
                        ));
                        if !self.existing_files_hashset.contains(&entry_string) {
                            self.existing_files_hashset.insert(entry_string.clone());
                            self.new_files_queue.push(entry_string.clone());
                        };
                    }
                }
            }
        }
        progress_bar.finish_with_message("Finished importing files");
        utility.print_function_timer();
    }

    pub fn print_generics(&self, utility: Utility) {
        Generic::print_generics(&self.generic_files, utility);
    }

    pub fn insert_episodes(&mut self, episodes: Vec<Episode>, utility: Utility) {
        let mut utility = utility.clone_add_location("insert_episodes(TV)");

        //find the associated show
        //insert episode into that show
        for episode in episodes {
            let show_uid = episode.show_uid;
            for show in &mut self.shows {
                if show.show_uid == show_uid {
                    show.insert_episode(episode, utility.clone());
                    break;
                }
            }
        }

        utility.print_function_timer();
    }

    pub fn ensure_show_exists(
        &mut self,
        show_title: String,
        utility: Utility,
        connection: &PgConnection,
    ) -> usize {
        let utility = utility.clone_add_location("ensure_show_exists(Show)");

        let show_uid = Show::show_exists(show_title.clone(), &self.shows, utility.clone());
        match show_uid {
            Some(uid) => uid,
            None => {
                if utility.preferences.print_shows || utility.preferences.show_output_whitelisted {
                    print(
                        Verbosity::INFO,
                        From::TV,
                        format!("Adding a new show: {}", show_title),
                        utility.preferences.show_output_whitelisted,
                        utility,
                    );
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

    pub fn print_shows(&self, utility: Utility) {
        let mut utility = utility.clone_add_location("print_shows(FileManager)");

        if !utility.preferences.print_shows {
            return;
        }
        for show in &self.shows {
            show.print_show(utility.clone());
            for season in &show.seasons {
                println!("S{} has {} episodes", season.number, season.episodes.len());
            }
        }

        utility.print_function_timer();
    }
}
