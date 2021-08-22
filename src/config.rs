use crate::manager::TrackedDirectories;
use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub allowed_extensions: Vec<String>,
    pub ignored_paths: Vec<String>,
    pub tracked_directories: TrackedDirectories,
}

impl Config {
    pub fn ensure_config_exists_then_get(preferences: &Preferences) -> Config {
        let config: Config;

        //Default config
        if Path::new(&preferences.config_file_path).exists() {
            let config_toml = fs::read_to_string(&preferences.config_file_path).unwrap();
            config = toml::from_str(&config_toml).unwrap();
        } else {
            let allowed_extensions = vec![
                String::from("mp4"),
                String::from("mkv"),
                String::from("webm"),
            ];
            let ignored_paths = vec![String::from(".recycle_bin")];
            let mut tracked_directories = TrackedDirectories::new();
            tracked_directories.root_directories = vec![String::from(r"D:\Desktop\tlmfiles")]; //these need to change
            config = Config {
                allowed_extensions,
                ignored_paths,
                tracked_directories,
            };
            let toml = toml::to_string(&config).unwrap();
            fs::write(&preferences.config_file_path, toml).unwrap();
        }

        return config;
    }
}

pub struct Preferences {
    pub default_print: bool,
    pub print_contents: bool,
    pub print_shows: bool,
    pub print_general: bool,
    pub config_file_path: String,
}

impl Preferences {
    pub fn new() -> Preferences {
        let mut prepare = Preferences {
            default_print: true,
            print_contents: false,
            print_shows: false,
            print_general: false,
            config_file_path: String::from("./.tlm_config"),
        };

        prepare.parse_arguments();

        return prepare;
    }

    pub fn parse_arguments(&mut self) {
        let mut parser = ArgumentParser::new();
        parser.set_description("tlm: Transcoding Library Manager");
        parser.refer(&mut self.default_print).add_option(
            &["--disable-print"],
            StoreFalse,
            "Disables printing by default. Specific types of print can be enabled on top of this",
        );
        parser.refer(&mut self.print_contents).add_option(
            &["--print-content"],
            StoreTrue,
            "Enable printing content",
        );
        parser.refer(&mut self.print_shows).add_option(
            &["--print-shows"],
            StoreTrue,
            "Enable printing shows",
        );
        parser.refer(&mut self.print_general).add_option(
            &["--print-general"],
            StoreTrue,
            "Enable printing general debug information",
        );
        parser.refer(&mut self.config_file_path).add_option(
            &["--config"],
            Store,
            "Set a custom config path",
        );

        parser.parse_args_or_exit();
    }
}
