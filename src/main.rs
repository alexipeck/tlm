use tlm::{database::{
        miscellaneous::db_purge,
        print::{print_contents, print_shows},
    }, manager::{FileManager, TrackedDirectories}, utility::Utility};
use argparse::{ArgumentParser, Store, StoreTrue, StoreFalse};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Deserialize, Serialize)]
struct Config {
    allowed_extensions: Vec<String>,
    ignored_paths: Vec<String>,
    tracked_directories: TrackedDirectories,
}

struct Options {
    default_print: bool,
    print_contents: bool,
    print_shows: bool,
    print_general: bool,
    db_purge: bool,
    config_file_path: String,
}

impl Options {
    pub fn new() -> Options {
        Options{
            default_print: true,
            print_contents: false,
            print_shows: false,
            print_general: false,
            db_purge: false,
            config_file_path: String::from("./.tlm_config")
        }
    }
}

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main");

    let mut options = Options::new();

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("tlm: Terminal Library Manager");
        parser
            .refer(&mut options.default_print)
            .add_option(&["--disable-print"], StoreFalse, "Disables printing by default. Specific types of print can be enabled on top of this");
        parser
            .refer(&mut options.db_purge)
            .add_option(&["--purge"], StoreTrue, "Purge database before starting");
        parser
            .refer(&mut options.print_contents)
            .add_option(&["--print-content"], StoreTrue, "Enable printing content");
        parser
            .refer(&mut options.print_shows)
            .add_option(&["--print-shows"], StoreTrue, "Enable printing shows");
        parser
            .refer(&mut options.print_general)
            .add_option(&["--print-general"], StoreTrue, "Enable printing general debug information");
        parser
            .refer(&mut options.config_file_path)
            .add_option(&["--config"], Store, "Set a custom config path");
    
        parser.parse_args_or_exit();
    }

    let config: Config;
    
    //Default config
    if Path::new(&options.config_file_path).exists() {
        let config_toml = fs::read_to_string(&options.config_file_path).unwrap();
        config = toml::from_str(&config_toml).unwrap();
    } else {
        let allowed_extensions = vec![String::from("mp4"), String::from("mkv"), String::from("webm"), String::from("MP4")];
        let ignored_paths = vec![String::from(".recycle_bin")];
        let mut tracked_directories = TrackedDirectories::new();
        tracked_directories.root_directories = vec![String::from(r"D:\Desktop\tlmfiles")];
        config = Config{allowed_extensions, ignored_paths, tracked_directories};
        let toml = toml::to_string(&config).unwrap();
        fs::write(&options.config_file_path, toml).unwrap();
    }




    if options.default_print || options.print_general {
        utility.enable_timing_print();
    }

    //purges the database, should be used selectively
    if options.db_purge {
        db_purge(utility.clone());
    }

    //A FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());


    file_manager.tracked_directories = config.tracked_directories;
    file_manager.import_files(&config.allowed_extensions, &config.ignored_paths);

    file_manager.process_new_files(
        utility.clone(),
    );

    utility.disable_timing_print();
    if options.default_print || options.print_contents {
        print_contents(file_manager.working_content.clone(), utility.clone());
    }
    if options.default_print || options.print_shows {
        print_shows(utility.clone());
    }
    
    //print_jobs(utility.clone());

    //queue.print();
    //shows.print();
}
