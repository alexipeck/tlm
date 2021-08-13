pub mod content;
pub mod database;
pub mod designation;
pub mod error_handling;
pub mod filter;
pub mod job;
pub mod print;
pub mod queue;
pub mod shows;
pub mod task;
pub mod utility;

use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::PathBuf,
    time::Instant,
};
//use utility;
use twox_hash::xxh3;
use walkdir::WalkDir;
use content::Content;
use database::insert::{
    insert_content,
    insert_episode_if_episode,
};
use utility::Utility;

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
}

pub fn handle_tracked_directories() -> TrackedDirectories {
    let mut tracked_directories = TrackedDirectories::new();

    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_directories
            .root_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/"));
        tracked_directories
            .root_directories
            .push_back(String::from(r"/home/alexi/tlm/test_files/"));
        tracked_directories
            .cache_directories
            .push_back(String::from(r"/home/anpeck/tlm/test_files/cache/"));
        tracked_directories
            .cache_directories
            .push_back(String::from(r"/home/alexi/tlm/test_files/cache/"));
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_directories.root_directories.push_back(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generics\",
        ));
        tracked_directories.root_directories.push_back(String::from(
            r"C:\Users\Alexi Peck\Desktop\tlm\test_files\shows\",
        ));
        tracked_directories
            .cache_directories
            .push_back(String::from(
                r"C:\Users\Alexi Peck\Desktop\tlm\test_files\cache\",
            ));
    }

    return tracked_directories;
}

pub fn process_new_files(new_files: Vec<PathBuf>, working_content: &mut Vec<Content>, utility: Utility) {
    let utility = utility.clone_and_add_location("process_new_files");

    for new_file in new_files {
        let mut content = Content::new(&new_file, utility.clone());
        content.set_uid(insert_content(content.clone(), utility.clone()));
        insert_episode_if_episode(content.clone(), utility.clone());
        working_content.push(content);
    }
}

//Hash set guarentees no duplicates in O(1) time
pub fn import_files(
    directories: &VecDeque<String>,
    allowed_extensions: &Vec<&str>,
    ignored_paths: &Vec<&str>,
    existing_files: &mut HashSet<PathBuf>,
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
    for directory in directories {
        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            if str_contains_strs(entry.path().to_str().unwrap(), ignored_paths) {
                break;
            }

            if entry.path().is_file() {
                if allowed_extensions.contains(&entry.path().extension().unwrap().to_str().unwrap())
                {
                    if !directory.contains("_encodeH4U8") {
                        //make entry into pathbuf into string
                        //check if string exists in existing_files
                        //if it doesn't, add it's hash to existing_files HashSet and to the filename_hash
                        let entry_string = entry.clone().into_path();
                        if !existing_files.contains(&entry_string) {
                            existing_files.insert(entry_string);
                            new_files.insert(entry.into_path());
                        };
                    }
                }
            }
        }
    }

    return new_files.iter().cloned().collect(); //return the set as a vector (this is not sorted but there are no duplicates)
}

pub fn get_show_title_from_pathbuf(pathbuf: &PathBuf) -> String {
    return pathbuf
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
}

pub fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
}
