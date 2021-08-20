use crate::{
    tv::TV,
    content::Content, database::ensure::ensure_tables_exist, utility::Utility,
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
            self.root_directories.push_back(String::from("T:\\"));
            
            /*self.root_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generics\",
            ));
            self.root_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\shows\",
            ));*/
            self.cache_directories.push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\cache\",
            ));
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

        file_manager.tracked_directories.add_manual_directories();

        let t = Content::get_all_contents(&mut file_manager.tv.working_shows, utility.clone());

        if t.len() < 1 {println!("fuck")};
        for t in &t {
            t.print(utility.clone());
        }

        file_manager.working_content = t;
        file_manager.existing_files_hashset = Content::get_all_filenames_as_hashset_from_contents(
            file_manager.working_content.clone(),
            utility.clone(),
        );

        return file_manager;
    }

    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(
        &mut self,
        allowed_extensions: &Vec<&str>,
        ignored_paths: &Vec<&str>,
    ) {
        //Return true if string contains any substring from Vector
        fn str_contains_strs(input_str: &str, substrings: &Vec<&str>) -> bool {
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
                if str_contains_strs(&entry.path().to_str().unwrap().to_lowercase(), ignored_paths) {
                    break;
                }
                if entry.path().is_file() {
                    if allowed_extensions.contains(&entry.path().extension().unwrap().to_str().unwrap()) {
                        //make entry into pathbuf into string
                        //check if string exists in existing_files
                        //if it doesn't, add it's hash to existing_files HashSet and to the filename_hash
                        let entry_string = entry.clone().into_path();
                        println!("{}", entry_string.to_string_lossy().to_string());
                        if !self.existing_files_hashset.contains(&entry_string) {
                            self.existing_files_hashset.insert(entry_string.clone());
                            self.new_files_queue.push(entry.clone().into_path());
                            println!("{}", entry_string.to_string_lossy().to_string());
                        };
                    }
                }
            }
        }
    }

    pub fn process_new_files(&mut self, utility: Utility) {
        let mut utility = utility.clone_and_add_location("process_new_files");
        utility.add_timer(0, "startup: processing new files");
        
        while self.new_files_queue.len() > 0 {
            let current = self.new_files_queue.pop();
            if current.is_some() {
                let current = current.unwrap();

                utility.add_timer(1, "startup: creating content from PathBuf");
    
                utility.add_timer(2, "startup: creating content from PathBuf");
                let mut content = Content::new(&current, &mut self.tv.working_shows, utility.clone());
                utility.store_timing_by_uid(2);
        
                utility.add_timer(3, "startup: creating content from PathBuf");
                content.set_uid(insert_content(content.clone(), utility.clone()));
                utility.store_timing_by_uid(3);
        
                utility.add_timer(4, "startup: creating content from PathBuf");
                insert_episode_if_episode(content.clone(), utility.clone());
                utility.store_timing_by_uid(4);
        
                self.working_content.push(content);
                utility.print_all_timers_except_one(0, 2, utility.clone());
            }
        }

        utility.print_specific_timer_by_uid(0, 1, utility.clone());
    }
}
