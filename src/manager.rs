use crate::{
    content::Content, database::ensure::ensure_tables_exist, show::Show, utility::Utility,
    database::insert::{insert_content, insert_episode_if_episode},
};
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct TrackedDirectories {
    pub root_directories: VecDeque<String>,
    pub cache_directories: VecDeque<String>,
}

impl TrackedDirectories {
    pub fn new() -> TrackedDirectories {
        TrackedDirectories {
            root_directories: VecDeque::new(),
            cache_directories: VecDeque::new(),
        }
    }

    pub fn add_manual_directories(&mut self) {
        if !cfg!(target_os = "windows") {
            //self.push(String::from("/mnt/nas/tvshows")); //manual entry
            self.root_directories
                .push_back(String::from(r"/home/anpeck/tlm/test_files/"));
            self.root_directories
                .push_back(String::from(r"/home/alexi/tlm/test_files/"));
            self.cache_directories
                .push_back(String::from(r"/home/anpeck/tlm/test_files/cache/"));
            self.cache_directories
                .push_back(String::from(r"/home/alexi/tlm/test_files/cache/"));
        } else {
            self.root_directories.push_back(String::from("D:\\Desktop\\tlmfiles"));
            /*self.root_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generics\",
            ));
            self.root_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\shows\",
            ));
            self.cache_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\cache\",
            ));*/
        }
    }
}

pub struct FileManager {
    //ordered by dependencies
    pub tracked_directories: TrackedDirectories,
    pub working_content: Vec<Content>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub working_shows: Vec<Show>,
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
        };

        file_manager.tracked_directories.add_manual_directories();
        file_manager.working_shows = Show::get_all_shows(utility.clone());
        file_manager.working_content =
            Content::get_all_contents(&mut file_manager.working_shows.clone(), utility.clone());
        file_manager.existing_files_hashset = Content::get_all_filenames_as_hashset_from_contents(
            file_manager.working_content.clone(),
            utility.clone(),
        );

        return file_manager;
    }

    pub fn process_new_files(&mut self,
        new_files: Vec<PathBuf>,
        utility: Utility,
    ) {
        let mut utility = utility.clone_and_add_location("process_new_files");
        utility.start_timer(0);
    
        for new_file in new_files {
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
    pub fn import_files(
        &mut self,
        allowed_extensions: &Vec<&str>,
        ignored_paths: &Vec<&str>,
    ) -> Vec<PathBuf> {
        //Return true if string contains any substring from Vector
        fn str_contains_strs(input_str: &str, substrings: &Vec<&str>) -> bool {
            for substring in substrings {
                if String::from(input_str).contains(substring) {
                    return true;
                }
            }
            return false;
        }

        let mut new_files = HashSet::new();

        //import all files in tracked root directories
        for directory in &self.tracked_directories.root_directories {
            for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
                if str_contains_strs(entry.path().to_str().unwrap(), ignored_paths) {
                    break;
                }

                if entry.path().is_file() {
                    if allowed_extensions
                        .contains(&entry.path().extension().unwrap().to_str().unwrap())
                    {
                        if !directory.contains("_encodeH4U8") {
                            //make entry into pathbuf into string
                            //check if string exists in existing_files
                            //if it doesn't, add it's hash to existing_files HashSet and to the filename_hash
                            let entry_string = entry.clone().into_path();
                            if !self.existing_files_hashset.contains(&entry_string) {
                                self.existing_files_hashset.insert(entry_string);
                                new_files.insert(entry.into_path());
                            };
                        }
                    }
                }
            }
        }

        return new_files.iter().cloned().collect(); //return the set as a vector (this is not sorted but there are no duplicates)
    }
}
