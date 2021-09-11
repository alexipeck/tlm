use crate::{
    config::Config,
    database::*,
    designation::Designation,
    generic::Generic,
    model::{NewEpisode, NewGeneric},
    print::{print, From, Verbosity},
    show::{Episode, Show},
    utility::{Utility, Traceback},
};
use diesel::pg::PgConnection;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use walkdir::{WalkDir, DirEntry};

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
        let mut utility = utility.clone_add_location(Traceback::NewFileManager);

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
        let mut utility = utility.clone_add_location(Traceback::AddExistingFilesToHashsetFileManager);
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
        let mut utility =
            utility.clone_add_location(Traceback::AddAllFilenamesToHashsetFileManager);
        for generic in generics {
            self.existing_files_hashset
                .insert(generic.full_path.clone());
        }

        utility.print_function_timer();
    }

    pub fn print_number_of_generics(&self, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::PrintNumberOfGenericsFileManager);

        print(
            Verbosity::INFO,
            From::Manager,
            format!(
                "Number of generic loaded in memory: {}",
                self.generic_files.len()
            ),
            false,
            utility.clone(),
        );

        utility.print_function_timer();
    }

    pub fn print_number_of_shows(&self, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::PrintNumberOfShowsFileManager);

        print(
            Verbosity::INFO,
            From::Manager,
            format!("Number of shows loaded in memory: {}", self.shows.len()),
            false,
            utility.clone(),
        );

        utility.print_function_timer();
    }

    pub fn print_number_of_episodes(&self, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::PrintNumberOfEpisodesFileManager);

        let mut episode_counter = 0;
        for show in &self.shows {
            for season in &show.seasons {
                episode_counter += season.episodes.len();
            }
        }

        print(
            Verbosity::INFO,
            From::Manager,
            format!("Number of episodes loaded in memory: {}", episode_counter),
            false,
            utility.clone(),
        );

        utility.print_function_timer();
    }

    pub fn process_new_files(&mut self, progress_bar: &ProgressBar, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::ProcessNewFilesFileManager);
        let connection = establish_connection();
        let mut new_episodes = Vec::new();
        let mut new_generics = Vec::new();
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
                new_generics.push(NewGeneric::new(
                    String::from(generic.full_path.to_str().unwrap()),
                    generic.designation as i32,
                    generic.profile,
                ));
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

            let show_uid =
                self.ensure_show_exists(show_title.clone(), utility.clone(), &connection);
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

        self.add_all_filenames_to_hashset_from_generics(&temp_generics, utility.clone());

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

        self.insert_episodes(episodes, utility.clone());

        progress_bar.finish();
        utility.print_function_timer();
    }

    pub fn path_contains_ignored_path(&self, input_str: &str, ignored_paths: &[String]) -> bool {
        for ignored_path in ignored_paths {
            if String::from(input_str).contains(&ignored_path.to_lowercase()) {
                return true;
            }
        }
        false
    }

    pub fn accept_file(&mut self, dir_entry: DirEntry, ignored_paths: &[String], allowed_extensions: &[String]) -> bool {
        let path = dir_entry.path();

        //rejects if the path contains any element of an ignored path
        for ignored_path in ignored_paths {
            if path.to_str().unwrap().to_lowercase().contains(&ignored_path.to_lowercase()) {
                return false;
            }
        }

        //rejects if the path isn't a file or doesn't have an extension
        if !dir_entry.path().is_file() || dir_entry.path().extension().is_none() {
            return false;
        }
        
        //rejects if the file doesn't have an allowed extension
        if !allowed_extensions.contains(&path.extension().unwrap().to_str().unwrap().to_lowercase()) {
            return false;
        }

        let entry_string = dir_entry.into_path();
        if !self.existing_files_hashset.contains(&entry_string) {
            self.existing_files_hashset.insert(entry_string.clone());
            self.new_files_queue.push(entry_string);
        };

        true
    }



    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(
        &mut self,
        allowed_extensions: &[String],
        ignored_paths: &[String],
        utility: Utility,
    ) {
        let mut utility = utility.clone_add_location(Traceback::ImportFilesFileManager);

        //import all files in tracked root directories
        for directory in &self.tracked_directories.root_directories.clone() {
            let entries = WalkDir::new(directory).into_iter().filter_map(|e| e.ok());
            for entry in entries {
                if !self.accept_file(entry, ignored_paths, allowed_extensions) {
                    //do something with rejected entry
                }
            }
        }
        utility.print_function_timer();
    }

    pub fn print_episodes(&self, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::PrintEpisodesFileManager);

        if !utility.preferences.print_episode && !utility.preferences.episode_output_whitelisted {
            return;
        }
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    episode.print_episode(utility.clone());
                }
            }
        }

        utility.print_function_timer();
    }

    pub fn print_generics(&self, utility: Utility) {
        Generic::print_generics(&self.generic_files, utility);
    }

    pub fn insert_episodes(&mut self, episodes: Vec<Episode>, utility: Utility) {
        let mut utility = utility.clone_add_location(Traceback::InsertEpisodesFileManager);
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
        let utility = utility.clone_add_location(Traceback::EnsureShowExistsFileManager);

        let show_uid = Show::show_exists(show_title.clone(), &self.shows, utility.clone());
        match show_uid {
            Some(uid) => uid,
            None => {
                if utility.preferences.print_shows || utility.preferences.show_output_whitelisted {
                    print(
                        Verbosity::INFO,
                        From::Manager,
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
        let mut utility = utility.clone_add_location(Traceback::PrintShowsFileManager);

        if !utility.preferences.print_shows {
            return;
        }
        for show in &self.shows {
            show.print_show(utility.clone());
            for season in &show.seasons {
                println!(
                    "S{:02} has {} episodes",
                    season.number,
                    season.episodes.len()
                );
            }
        }

        utility.print_function_timer();
    }
}
