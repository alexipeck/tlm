use crate::{
    content::Content, database::ensure::ensure_tables_exist, show::Show, utility::Utility,
};
use std::{
    collections::{HashSet, VecDeque},
    path::PathBuf,
};

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
    pub existing_files_hashset: Option<HashSet<PathBuf>>,
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
            existing_files_hashset: None,
        };

        file_manager.tracked_directories.add_manual_directories();
        file_manager.working_shows = Show::get_all_shows(utility.clone());
        file_manager.working_content =
            Content::get_all_contents(&mut file_manager.working_shows.clone(), utility.clone());
        file_manager.existing_files_hashset =
            Some(Content::get_all_filenames_as_hashset_from_contents(
                file_manager.working_content.clone(),
                utility.clone(),
            ));

        return file_manager;
    }
}
