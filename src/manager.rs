use crate::{
    content::Content,
    database::ensure::ensure_tables_exist,
    database::insert::{insert_content, insert_episode_if_episode},
    tv::TV,
    utility::Utility,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

use crate::model::*;
use crate::diesel::prelude::*;
use crate::schema::content::dsl::*;
use crate::{establish_connection, create_content, create_episode};

#[derive(Default, Debug, Deserialize, Serialize)]
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
    //ordered by dependencies
    pub tracked_directories: TrackedDirectories,
    pub working_content: Vec<Content>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub tv: TV,
    pub new_files_queue: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(utility: Utility) -> FileManager {
        let utility = utility.clone_and_add_location("new_file_manager");

        ensure_tables_exist(utility.clone());

        let mut file_manager = FileManager {
            tracked_directories: TrackedDirectories::new(),
            tv: TV::new(utility.clone()),
            working_content: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
        };

        file_manager.working_content =
            Content::get_all_contents(&mut file_manager.tv.working_shows.clone(), utility.clone());
        file_manager.existing_files_hashset = Content::get_all_filenames_as_hashset_from_contents(
            file_manager.working_content.clone(),
            utility.clone(),
        );

        return file_manager;
    }

    pub fn process_new_files(&mut self, utility: Utility) {
        let mut utility = utility.clone_and_add_location("process_new_files");
        let connection = establish_connection();
        utility.add_timer(0, "startup: processing new files");
        println!("WHATHTEFUCK{}", self.new_files_queue.len());
        while self.new_files_queue.len() > 0 {
            let current = self.new_files_queue.pop();
            if current.is_some() {
                let current = current.unwrap();

                utility.add_timer(1, "startup: dealing with content from PathBuf");

                utility.add_timer(2, "startup: creating content from PathBuf");
                let mut c =
                    Content::new(&current, &mut self.tv.working_shows, utility.clone());
                utility.store_timing_by_uid(2);

                utility.add_timer(3, "startup: inserting content to DB");
                let content_model = create_content(&connection, String::from(c.full_path.to_str().unwrap()), c.designation as i32);
                c.content_uid = Some(content_model.content_uid as usize);
                utility.store_timing_by_uid(3);

                utility.add_timer(4, "startup: inserting episode to DB if it is such");
                if c.content_is_episode() {
                    let c_uid = c.content_uid.unwrap() as i32;
                    let s_uid = c.show_uid.unwrap() as i32;
                    let (season_number_temp, episode_number_temp) = c.show_season_episode.as_ref().unwrap();
                    let season_number = *season_number_temp as i16;
                    let episode_number = episode_number_temp[0] as i16;
                    create_episode(&connection, c_uid, s_uid, c.show_title.as_ref().unwrap().to_string(), season_number as i32, episode_number as i32);
                }
                
                //insert_episode_if_episode(c.clone(), utility.clone());
                utility.store_timing_by_uid(4);

                self.working_content.push(c);
                utility.print_specific_timer_by_uid(1, 2, utility.clone());
                utility.print_all_timers_except_many(vec![0, 1], 3, utility.clone());
                utility.delete_or_reset_multiple_timers(false, vec![1, 2, 3, 4]);
            }
        }

        utility.print_specific_timer_by_uid(0, 1, utility.clone());
    }

    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(&mut self, allowed_extensions: &Vec<String>, ignored_paths: &Vec<String>) {
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
                        let entry_string = entry.clone().into_path();
                        if !self.existing_files_hashset.contains(&entry_string) {
                            self.existing_files_hashset.insert(entry_string.clone());
                            self.new_files_queue.push(entry.clone().into_path());
                        };
                    }
                }
            }
        }
    }
}
