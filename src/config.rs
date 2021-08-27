use crate::manager::TrackedDirectories;
use crate::print::{print, From, Verbosity};
use crate::utility::Utility;
use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

///This struct contains any system specific data (paths, extensions, etc)
/// likely will be replaced later with database tables but as we clear data
/// so often I would prefer this config file for now.
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub allowed_extensions: Vec<String>,
    pub ignored_paths: Vec<String>,
    pub tracked_directories: TrackedDirectories,
}

impl Config {
    ///Config constructor loads the config from the path defined at the cli
    /// or if it doesn't exist creates a default config file
    pub fn new(preferences: &Preferences) -> Config {
        let utility = Utility::new("new (Config)", 30);
        let config: Config;

        if Path::new(&preferences.config_file_path).exists() {
            let config_toml = match fs::read_to_string(&preferences.config_file_path) {
                Ok(x) => x,
                Err(err) => {
                    print(
                        Verbosity::CRITICAL,
                        From::Config,
                        utility,
                        format!("Failed to read config file: {}", err),
                        false,
                    );
                    panic!();
                }
            };
            config = match toml::from_str(&config_toml) {
                Ok(x) => x,
                Err(err) => {
                    print(
                        Verbosity::CRITICAL,
                        From::Config,
                        utility,
                        format!("Failed to parse toml: {}", err),
                        false,
                    );
                    panic!();
                }
            };
        } else {
            //Default config
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
            if fs::write(&preferences.config_file_path, toml).is_err() {
                print(
                    Verbosity::CRITICAL,
                    From::Config,
                    utility,
                    String::from("Failed to write config file"),
                    false,
                );
                panic!();
            }
        }

        return config;
    }
}

///Helper struct to make passing data for command line arguments easier
#[derive(Clone, Debug)]
pub struct Preferences {
    pub default_print: bool,
    pub print_contents: bool,
    pub print_shows: bool,
    pub print_general: bool,
    pub config_file_path: String,
    pub min_verbosity: String,

    pub timing_enabled: bool,
    pub timing_threshold: u128,
}

impl Preferences {
    pub fn new() -> Preferences {
        let mut prepare = Preferences {
            default_print: true,
            print_contents: false,
            print_shows: false,
            print_general: false,
            config_file_path: String::from("./.tlm_config"),
            min_verbosity: String::from("INFO"),

            timing_enabled: false,
            timing_threshold: 0,
        };

        prepare.parse_arguments();

        return prepare;
    }

    ///Parses command line arguments using arg parse
    fn parse_arguments(&mut self) {
        let mut parser = ArgumentParser::new();
        parser.set_description("tlm: Transcoding Library Manager");
        parser.refer(&mut self.default_print).add_option(
            &["--disable-print", "--no-print"],
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
            &["--config", "-c"],
            Store,
            "Set a custom config path",
        );
        parser.refer(&mut self.min_verbosity).add_option(
            &["--min-severity", "--min-verbosity"],
            Store,
            "Set a minimum severity (debug, info, warning, error, critical)",
        );

        parser.refer(&mut self.timing_enabled).add_option(
            &["--enable-timing"],
            StoreTrue,
            "Enable program self-timing",
        );

        parser.refer(&mut self.timing_threshold).add_option(
            &["--timing-threshold", "--timing-cutoff"],
            Store,
            "Threshold for how slow a timed event has to be in order to print",
        );

        parser.parse_args_or_exit();
    }
}
