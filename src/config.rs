//!Set of functions and structures to make is easier to handle the config file
//!and command line arguments
use crate::file_manager::TrackedDirectories;
use argparse::{ArgumentParser, Store, StoreFalse, StoreOption, StoreTrue};
use directories::BaseDirs;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tracing::debug;
use tracing::error;

///This struct contains any system specific data (paths, extensions, etc)
/// likely will be replaced later with database tables but as we clear data
/// so often I would prefer this config file for now.
#[derive(Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub allowed_extensions: Vec<String>,
    pub ignored_paths: Vec<String>,
    #[serde(skip)]
    pub ignored_paths_regex: Vec<Regex>,
    pub tracked_directories: TrackedDirectories,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerConfig {
    pub server_address: String,
    pub server_port: u16,
    pub uid: Option<u32>,
    config_path: PathBuf,
}

impl fmt::Display for WorkerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ws://{}:{}", self.server_address, self.server_port)
    }
}

impl WorkerConfig {
    pub fn new(config_path: PathBuf) -> WorkerConfig {
        let config: WorkerConfig;

        if config_path.exists() {
            let config_toml = match fs::read_to_string(config_path) {
                Ok(config_toml) => config_toml,
                Err(err) => {
                    error!("Failed to read config file: {}", err);
                    panic!();
                }
            };
            config = match toml::from_str(&config_toml) {
                Ok(config_toml) => config_toml,
                Err(err) => {
                    error!("Failed to parse toml: {}", err);
                    panic!();
                }
            };
        } else {
            //Default config
            config = WorkerConfig {
                server_address: "127.0.0.1".to_string(),
                server_port: 8888,
                uid: None,
                config_path: config_path.clone(),
            };
            let toml = toml::to_string(&config).unwrap();
            if fs::write(config_path.clone(), toml).is_err() {
                error!(
                    "Failed to write config file at: {}",
                    config_path.to_string_lossy()
                );
                panic!();
            }
        }
        config
    }

    pub fn insert_uid(&mut self, uid: u32) {
        self.uid = Some(uid);
    }

    pub fn update_config_on_disk(&self) {
        let toml = toml::to_string(&self).unwrap();
        if fs::write(self.config_path.clone(), toml).is_err() {
            error!(
                "Failed to write config file at: {}",
                self.config_path.to_string_lossy()
            );
            panic!();
        }
    }
}

impl ServerConfig {
    ///Config constructor loads the config from the path defined at the cli
    /// or if it doesn't exist creates a default config file
    pub fn new(preferences: &Preferences) -> ServerConfig {
        let mut config: ServerConfig;

        if Path::new(&preferences.config_file_path).exists() {
            let config_toml = match fs::read_to_string(&preferences.config_file_path) {
                Ok(config_toml) => config_toml,
                Err(err) => {
                    error!("Failed to read config file: {}", err);
                    panic!();
                }
            };
            config = match toml::from_str(&config_toml) {
                Ok(config_toml) => config_toml,
                Err(err) => {
                    error!("Failed to parse toml: {}", err);
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
            tracked_directories.root_directories = vec![String::from(
                r"C:\\Users\\Alexi Peck\\Desktop\\tlm\\test_files",
            )]; //these need to change
            config = ServerConfig {
                port: 8888,
                allowed_extensions,
                ignored_paths,
                ignored_paths_regex: Vec::new(),
                tracked_directories,
            };
            let toml = toml::to_string(&config).unwrap();
            if fs::write(&preferences.config_file_path, toml).is_err() {
                error!(
                    "Failed to write config file at: {}",
                    preferences.config_file_path
                );
                panic!();
            }
        }

        if preferences.port.is_some() {
            config.port = preferences.port.unwrap();
        }
        for ignored_path in &config.ignored_paths {
            config
                .ignored_paths_regex
                .push(Regex::new(&format!("(?i){}", regex::escape(ignored_path))).unwrap())
        }

        config
    }
}

///Helper struct to make passing data for command line arguments easier
#[derive(Clone, Debug)]
pub struct Preferences {
    pub default_print: bool,
    pub print_generic: bool,
    pub print_shows: bool,
    pub print_episode: bool,
    pub print_general: bool,
    pub config_file_path: String,
    pub timing_enabled: bool,
    pub timing_threshold: u128,
    pub port: Option<u16>,
    pub generic_output_whitelisted: bool,
    pub show_output_whitelisted: bool,
    pub episode_output_whitelisted: bool,
    pub disable_input: bool,
}
impl Default for Preferences {
    fn default() -> Preferences {
        let base_dirs = BaseDirs::new().unwrap_or_else(|| {
            error!("Home directory could not be found");
            panic!();
        });
        let config_path = base_dirs.config_dir().join("tlm/tlm_server.config");
        let mut prepare = Preferences {
            default_print: true,
            print_generic: false,
            print_shows: false,
            print_episode: false,
            print_general: false,
            config_file_path: String::from(config_path.to_str().unwrap()),
            timing_enabled: false,
            timing_threshold: 0,
            port: None,

            generic_output_whitelisted: false,
            show_output_whitelisted: false,
            episode_output_whitelisted: false,
            disable_input: false,
        };

        prepare.parse_arguments();

        prepare
    }
}

impl Preferences {
    ///Parses command line arguments using arg parse
    fn parse_arguments(&mut self) {
        let mut parser = ArgumentParser::new();
        parser.set_description("tlm: Transcoding Library Manager");

        parser.refer(&mut self.default_print).add_option(
            &["--disable-print", "--no-print"],
            StoreFalse,
            "Disables printing by default. Specific types of print can be enabled on top of this",
        );

        parser.refer(&mut self.print_generic).add_option(
            &["--print-generic"],
            StoreTrue,
            "Enable printing generic",
        );

        parser.refer(&mut self.print_shows).add_option(
            &["--print-shows"],
            StoreTrue,
            "Enable printing shows",
        );

        parser.refer(&mut self.print_episode).add_option(
            &["--print-episodes"],
            StoreTrue,
            "Enable printing episodes",
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

        parser.refer(&mut self.generic_output_whitelisted).add_option(
            &["--whitelist-generic-output"],
            StoreTrue,
            "Whitelist all output from generic, whitelisting a type will cause it to print regardless of other limiting flags",
        );

        parser.refer(&mut self.show_output_whitelisted).add_option(
            &["--whitelist-show-output"],
            StoreTrue,
            "Whitelist all output from shows, whitelisting a type will cause it to print regardless of other limiting flags",
        );

        parser.refer(&mut self.disable_input).add_option(
            &["--disable-input", "--no-input"],
            StoreTrue,
            "Don't accept any inputs from the user (Testing only will be removed later)",
        );

        parser.refer(&mut self.port).add_option(
            &["--port", "-p"],
            StoreOption,
            "Overwrite the port set in the config",
        );

        parser.parse_args_or_exit();
    }
}
