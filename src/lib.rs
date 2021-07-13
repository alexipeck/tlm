pub mod content;
pub mod designation;
pub mod queue;

use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use twox_hash::xxh3;
use walkdir::WalkDir;

pub fn import_files(
    file_paths: &mut Vec<PathBuf>,
    directories: &VecDeque<String>,
    allowed_extensions: &Vec<&str>,
    ignored_paths: &Vec<&str>,
) {
    //Return true if string contains any substring from Vector
    fn str_contains_strs(input_str: &str, substrings: &Vec<&str>) -> bool {
        for substring in substrings {
            if String::from(input_str).contains(substring) {
                return true;
            }
        }
        false
    }

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
                        file_paths.push(entry.into_path());
                    }
                }
            }
        }
    }
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

pub mod print {
    //trickle up
    pub enum Verbosity {
        CRITICAL = 1,
        ERROR = 2,
        WARNING = 3,
        INFO = 4,
        DEBUG = 5,
        NOTSET = 0,
    }

    pub fn print(verbosity: Verbosity, called_from: &str, string: String) {
        //print(Verbosity::DEBUG, "", format!(""));
        let set_output_verbosity_level = Verbosity::DEBUG as usize; //would be set as a filter in any output view

        let current_verbosity_level = verbosity as usize;
        let verbosity_string: String;
        match current_verbosity_level {
            1 => verbosity_string = "CRITICAL".to_string(),
            2 => verbosity_string = "ERROR".to_string(),
            3 => verbosity_string = "WARNING".to_string(),
            4 => verbosity_string = "INFO".to_string(),
            5 => verbosity_string = "DEBUG".to_string(),
            _ => verbosity_string = "NOTSET".to_string(),
        }

        if current_verbosity_level <= set_output_verbosity_level {
            println!("[{}][{}] {}", verbosity_string, called_from, string);
        }
    }
}
