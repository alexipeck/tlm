pub mod content;
pub mod designation;
pub mod queue;
pub mod task;

use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    time::Instant,
};
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
    #[derive(Clone, Debug, PartialEq)]
    pub enum Verbosity {
        CRITICAL = 1,
        ERROR = 2,
        WARNING = 3,
        INFO = 4,
        DEBUG = 5,
        NOTSET = 0,
    }

    pub enum From {
        NOTSET = 0,
        Main = 1,
        Lib = 2,
        Content = 3,
        Shows = 4,
        Queue = 5,
        DB = 6,
        Job = 7,
    }

    pub fn convert_function_callback_to_string(input: Vec<&str>) -> String {
        let mut call_functions_string: String = String::new();
        let mut single_execute_done = false;
        for function in &input {
            if !single_execute_done {
                call_functions_string += &format!("{}", function);
                single_execute_done = true;
            } else {
                call_functions_string += &format!(" > {}", function);
            }
        }
        return call_functions_string;
    }

    pub fn print(
        verbosity: Verbosity,
        from_module: From,
        call_functions: Vec<&str>,
        string: String,
    ) {
        fn print(
            verbosity_string: &str,
            from_module_string: &str,
            call_functions_string: String,
            string: String,
        ) {
            println!(
                "[{}][{}][{}] {}",
                verbosity_string, from_module_string, call_functions_string, string
            );
        }

        //print(Verbosity::DEBUG, r"", format!(""));
        let set_output_verbosity_level = Verbosity::DEBUG as usize; //would be set as a filter in any output view
        let show_only = Verbosity::DEBUG;

        //module called from
        let from_module_string: &str;
        match from_module as usize {
            1 => from_module_string = "main",
            2 => from_module_string = "lib",
            3 => from_module_string = "content",
            4 => from_module_string = "shows",
            5 => from_module_string = "queue",
            6 => from_module_string = "db",
            7 => from_module_string = "job",
            _ => from_module_string = "notset",
        }

        //verbosity
        let current_verbosity_level = verbosity.clone() as usize;
        let verbosity_string: &str;
        match current_verbosity_level {
            1 => verbosity_string = "CRITICAL",
            2 => verbosity_string = "ERROR",
            3 => verbosity_string = "WARNING",
            4 => verbosity_string = "INFO",
            5 => verbosity_string = "DEBUG",
            _ => verbosity_string = "NOTSET",
        }

        //called from
        let mut call_functions_string: String = String::new();
        if verbosity.clone() as usize == Verbosity::CRITICAL as usize
            || verbosity.clone() as usize == Verbosity::ERROR as usize
        {
            call_functions_string = convert_function_callback_to_string(call_functions);
        } else {
            call_functions_string += &format!("{}", call_functions[call_functions.len() - 1]);
        }

        if verbosity == show_only {
            print(
                verbosity_string,
                from_module_string,
                call_functions_string,
                string,
            );
        } else if current_verbosity_level <= set_output_verbosity_level {
            print(
                verbosity_string,
                from_module_string,
                call_functions_string,
                string,
            );
        }
    }
}
