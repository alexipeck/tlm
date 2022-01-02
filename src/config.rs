//!Set of functions and structures to make is easier to handle the config file
//!and command line arguments
use crate::file_manager::TrackedDirectories;
use crate::pathbuf_to_string;
use argparse::{ArgumentParser, Store, StoreOption, StoreTrue};
use directories::BaseDirs;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tracing::error;
use tracing::warn;

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

impl ServerConfig {
    pub fn default() -> Self {
        let allowed_extensions = vec![
            String::from("mp4"),
            String::from("mkv"),
            String::from("webm"),
        ];
        let ignored_paths = vec![String::from(".recycle_bin")];
        let mut tracked_directories = TrackedDirectories::default();
        //TODO: Remove hardcoding
        tracked_directories.add_root_directory(PathBuf::from(
            r"C:\\Users\\Alexi Peck\\Desktop\\tlm\\test_files".to_string(),
        ));
        //TODO: Have a better method for getting a temp network share directory from the user
        tracked_directories
            .assign_global_temp_directory(&PathBuf::from(r"\\192.168.2.30\tlm_temp\"));
        Self {
            port: 8888,
            allowed_extensions,
            ignored_paths,
            ignored_paths_regex: Vec::new(),
            tracked_directories,
        }
    }

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
                Ok(config) => {
                    let config: ServerConfig = config;
                    //TODO: All paths from the config file should be checked and cache directories need to have ensure_path_exists to account for child folders in temp
                    ensure_path_exists(config.tracked_directories.get_cache_directory());
                    config
                },
                Err(err) => {
                    error!("Failed to parse toml: {}", err);
                    panic!();
                }
            };
        } else {
            config = ServerConfig::default();
            let toml = toml::to_string(&config).unwrap();
            if fs::write(&preferences.config_file_path, toml).is_err() {
                error!(
                    "Failed to write config file at: {}",
                    preferences.config_file_path
                );
                panic!();
            }
        }

        //TODO: Need to check that the server config from the config file is actually complete
        if !config.tracked_directories.has_cache_directory() {
            config.tracked_directories.assign_cache_directory(None);
            warn!("Used default cache directory instead of path specified in config file");
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

#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerConfig {
    pub server_address: String,
    pub server_port: u16,
    pub uid: Option<i32>,
    #[serde(skip)]
    config_path: PathBuf,
    pub temp_path: PathBuf,
}

impl WorkerConfig {
    pub fn new(config_path: PathBuf) -> WorkerConfig {
        let mut config: WorkerConfig;

        if config_path.exists() {
            let config_toml = match fs::read_to_string(config_path.clone()) {
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
                temp_path: env::temp_dir(),
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
        config.config_path = config_path;
        config
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

impl fmt::Display for WorkerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ws://{}:{}", self.server_address, self.server_port)
    }
}

///Helper struct to make passing data for command line arguments easier
#[derive(Clone, Debug)]
pub struct Preferences {
    pub config_file_path: String,
    pub disable_input: bool,
    pub file_system_read_only: bool,
    pub port: Option<u16>,
    pub silent: bool,
}
impl Default for Preferences {
    fn default() -> Preferences {
        let base_dirs = BaseDirs::new().unwrap_or_else(|| {
            error!("Home directory could not be found");
            panic!();
        });
        let config_path = base_dirs.config_dir().join("tlm/tlm_server.config");
        let mut prepare = Preferences {
            config_file_path: pathbuf_to_string(&config_path),
            disable_input: false,
            file_system_read_only: false,
            port: None,
            silent: false,
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

        parser.refer(&mut self.config_file_path).add_option(
            &["-c", "--config"],
            Store,
            "Set a custom config path",
        );

        parser.refer(&mut self.disable_input).add_option(
            &["--disable-input", "--no-input"],
            StoreTrue,
            "Don't accept any inputs from the user",
        );

        //TODO: Any file system actions need a check, if not making an action negatively affects too much, discard this option
        parser.refer(&mut self.file_system_read_only).add_option(
            &["--fsro", "--fs_read_only", "--file_system_read_only"],
            StoreTrue,
            "Don't overwrite any media files in the file system",
        );

        parser.refer(&mut self.port).add_option(
            &["-p", "--port"],
            StoreOption,
            "Overwrite the port set in the config",
        );

        //TODO: Add a check on every tracing output code block, or use some fancy thing that tracing includes
        parser.refer(&mut self.silent).add_option(
            &["-s", "--silent"],
            StoreTrue,
            "Disables output",
        );

        parser.parse_args_or_exit();
    }
}
