use crate::{
    content::Content,
    database::ensure::ensure_tables_exist,
    database::insert::{insert_content, insert_episode_if_episode},
    show::Show,
    utility::Utility,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

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
    pub working_shows: Vec<Show>,
    pub new_files_queue: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(utility: Utility) -> FileManager {
        let utility = utility.clone_and_add_location("new_file_manager");

        ensure_tables_exist(utility.clone());

        let mut file_manager = FileManager {
            tracked_directories: TrackedDirectories::new(),
            working_shows: Vec::new(),
            working_content: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
        };

        file_manager.working_shows = Show::get_all_shows(utility.clone());
        file_manager.working_content =
            Content::get_all_contents(&mut file_manager.working_shows.clone(), utility.clone());
        file_manager.existing_files_hashset = Content::get_all_filenames_as_hashset_from_contents(
            file_manager.working_content.clone(),
            utility.clone(),
        );

        return file_manager;
    }

    pub fn process_new_files(&mut self, utility: Utility) {
        let mut utility = utility.clone_and_add_location("process_new_files");
        utility.start_timer(0);

        for new_file in &self.new_files_queue {
            utility.start_timer(1);

            utility.start_timer(2);
            let mut content = Content::new(&new_file, &mut self.working_shows, utility.clone());
            utility.save_timing(2, utility.clone());

            utility.start_timer(3);
            content.set_uid(insert_content(content.clone(), utility.clone()));
            utility.save_timing(3, utility.clone());

            utility.start_timer(4);
            insert_episode_if_episode(content.clone(), utility.clone());
            utility.save_timing(4, utility.clone());

            self.working_content.push(content);
            utility.print_timer_from_stage_and_task(
                1,
                "startup",
                "creating content from PathBuf",
                1,
                utility.clone(),
            );
            utility.print_timer_from_stage_and_task_from_saved(
                2,
                "startup",
                "creating content from PathBuf",
                2,
                utility.clone(),
            );
            utility.print_timer_from_stage_and_task_from_saved(
                3,
                "startup",
                "inserting content to the database",
                2,
                utility.clone(),
            );
            utility.print_timer_from_stage_and_task_from_saved(
                4,
                "startup",
                "inserting episode to the database",
                2,
                utility.clone(),
            );
        }
        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "processing new files",
            0,
            utility.clone(),
        );
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
                    let temp_string =
                        String::from(entry.path().extension().unwrap().to_str().unwrap());
                    if allowed_extensions.contains(&temp_string) {
                        if !directory.contains("_encodeH4U8") {
                            //make entry into pathbuf into string
                            //check if string exists in existing_files
                            //if it doesn't, add it's hash to existing_files HashSet and to the filename_hash
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
}
