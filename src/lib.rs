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
pub mod utility;

use content::Content;
use database::insert::{insert_content, insert_episode_if_episode};
use show::Show;
use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::PathBuf,
    time::Instant,
};
use twox_hash::xxh3;
use utility::Utility;
use walkdir::WalkDir;

pub fn process_new_files(
    new_files: Vec<PathBuf>,
    working_content: &mut Vec<Content>,
    working_shows: &mut Vec<Show>,
    utility: Utility,
) {
    let mut utility = utility.clone_and_add_location("process_new_files");
    utility.start_timer(0);

    for new_file in new_files {
        utility.start_timer(1);

        utility.start_timer(2);
        let mut content = Content::new(&new_file, working_shows, utility.clone());
        utility.save_timing(2, utility.clone());

        utility.start_timer(3);
        content.set_uid(insert_content(content.clone(), utility.clone()));
        utility.save_timing(3, utility.clone());

        utility.start_timer(4);
        insert_episode_if_episode(content.clone(), utility.clone());
        utility.save_timing(4, utility.clone());

        working_content.push(content);
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

pub fn load_from_database(utility: Utility) -> (Vec<Content>, Vec<Show>, HashSet<PathBuf>) {
    let utility = utility.clone_and_add_location("load_from_database");

    let mut working_shows: Vec<Show> = Show::get_all_shows(utility.clone());

    let working_content = Content::get_all_contents(&mut working_shows, utility.clone());

    let existing_files_hashset: HashSet<PathBuf> =
        Content::get_all_filenames_as_hashset_from_contents(
            working_content.clone(),
            utility.clone(),
        );

    return (working_content, working_shows, existing_files_hashset);
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
