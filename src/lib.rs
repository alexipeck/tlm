pub mod config;
pub mod content;
pub mod database;
pub mod designation;
pub mod error_handling;
pub mod job;
pub mod manager;
pub mod print;
pub mod queue;
pub mod show;
pub mod task;
pub mod timer;
pub mod tv;
pub mod utility;

use content::Content;
use std::{collections::HashSet, fs, path::PathBuf, time::Instant};
use tv::Show;
use twox_hash::xxh3;
use utility::Utility;

pub fn load_from_database(utility: Utility) -> (Vec<Content>, Vec<Show>, HashSet<PathBuf>) {
    let utility = utility.clone_and_add_location("load_from_database");

    let mut working_shows: Vec<Show> = Show::get_all_shows(utility.clone());
    let working_content = Content::get_all_contents(&mut working_shows, utility.clone());
    let existing_files_hashset: HashSet<PathBuf> =
        Content::get_all_filenames_as_hashset_from_content(
            working_content.clone(),
            utility.clone(),
        );

    return (working_content, working_shows, existing_files_hashset);
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
