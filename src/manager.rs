use crate::model::{NewEpisode, NewGeneric};
use crate::{
    config::Config,
    database::{create_episodes, create_generics, establish_connection, get_all_generics},
    designation::Designation,
    generic::Generic,
    print::{print, From, Verbosity},
    tv::{Show, TV},
    utility::Utility,
};
use diesel::PgConnection;
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
    pub working_content: Vec<Generic>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub tv: TV,
    pub new_files_queue: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(config: &Config, utility: Utility) -> FileManager {
        let mut utility = utility.clone_add_location("new(FileManager)");

        let mut file_manager = FileManager {
            tracked_directories: TrackedDirectories::new(),
            tv: TV::new(utility.clone()),
            working_content: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
        };

        file_manager.working_content = file_manager.get_all_content(utility.clone());
        file_manager.existing_files_hashset = Generic::get_all_filenames_as_hashset_from_generics(
            &file_manager.working_content,
            utility.clone(),
        );
        file_manager.tracked_directories = config.tracked_directories.clone();

        utility.print_function_timer();
        return file_manager;
    }

    pub fn print_number_of_content(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_number_of_content(FileManager)");

        print(
            Verbosity::INFO,
            From::Manager,
            format!(
                "Number of content loaded in memory: {}",
                self.working_content.len()
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
            format!("Number of shows loaded in memory: {}", self.tv.shows.len()),
            false,
            utility,
        );
    }

    pub fn get_all_content(&mut self, utility: Utility) -> Vec<Generic> {
        let mut utility = utility.clone_add_location("get_all_contents(Generic)");

        let mut content: Vec<Generic> = Vec::new();
        let connection = establish_connection();
        let raw_content = get_all_generics(utility.clone());

        for content_model in raw_content {
            content.push(Generic::from_generic_model(
                content_model,
                &mut self.tv.shows,
                utility.clone(),
                &connection,
            ));
        }

        utility.print_function_timer();
        return content;
    }

    pub fn process_new_files(&mut self, utility: Utility) {
        let mut utility = utility.clone_add_location("process_new_files(FileManager)");
        let connection = establish_connection();
        let mut new_episodes = Vec::new();
        let mut new_generics = Vec::new();

        //Temporary because we need to get the id's that the database returns
        //Will just be appended to working content at the end
        let mut temp_generics = Vec::new();

        //Create Generic and NewGeneric that will be added to the database in a batch
        while self.new_files_queue.len() > 0 {
            let current = self.new_files_queue.pop();
            if current.is_some() {
                let current = current.unwrap();

                let generic = Generic::new(&current, utility.clone());
                new_generics.push(NewGeneric {
                    full_path: String::from(generic.full_path.to_str().unwrap()),
                    designation: generic.designation as i32,
                });

                temp_generics.push(generic);
            }
        }

        //Insert the content and then update the uid's for the full Generic structure
        let contents = create_generics(&connection, new_generics);
        for i in 0..contents.len() {
            temp_generics[i].generic_uid = Some(contents[i].id as usize);
        }
        self.working_content.append(&mut temp_generics);

        //Build all the NewEpisodes so we can do a batch insert that is faster than doing one at a time in a loop
        for generic in &temp_generics {
            let show_uid = generic.show_uid.unwrap() as i32;
            let (season_number_temp, episode_number_temp) =
                generic.show_season_episode.as_ref().unwrap();
            let season_number = *season_number_temp as i16;
            let episode_number = episode_number_temp[0] as i16;

            let new_episode = NewEpisode::new(
                generic.get_generic_uid(utility.clone()),
                0,              //show_uid
                "".to_string(), //show_title
                "".to_string(), //episode_title
                season_number,
                episode_number,
            );
            new_episodes.push(new_episode);
        }

        //episodes isn't being used yet but this does insert into the database
        let _episodes = create_episodes(&connection, new_episodes);
        utility.print_function_timer();
    }

    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(
        &mut self,
        allowed_extensions: &Vec<String>,
        ignored_paths: &Vec<String>,
        utility: Utility,
    ) {
        let mut utility = utility.clone_add_location("import_files(FileManager)");

        //Return true if string contains any substring from Vector
        fn str_contains_strs(input_str: &str, substrings: &Vec<String>) -> bool {
            for substring in substrings {
                if String::from(input_str).contains(&substring.to_lowercase()) {
                    return true;
                }
            }
            return false;
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
                        if !self.existing_files_hashset.contains(&entry_string) {
                            self.existing_files_hashset.insert(entry_string.clone());
                            self.new_files_queue.push(entry_string.clone());
                        };
                    }
                }
            }
        }

        utility.print_function_timer();
    }

    pub fn print_shows(&self, utility: Utility) {
        Show::print_shows(&self.tv.working_shows, utility.clone());
    }

    pub fn print_content(&self, utility: Utility) {
        Generic::print_content(&self.working_content, utility.clone());
    }
}
