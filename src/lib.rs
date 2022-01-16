#![doc = include_str!("../README.md")]

use generic::FileVersion;
use tracing::warn;
use {
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        ffi::OsStr,
        fs::{self, copy as fs_copy, remove_file as fs_remove_file, File},
        io::{Error, Write},
        net::SocketAddr,
        path::{Path, PathBuf},
    },
    futures_channel::mpsc::UnboundedSender,
    tracing::error,
    tokio_tungstenite::tungstenite::Message,
    worker::WorkerMessage,
};

pub mod config;
pub mod database;
pub mod debug;
pub mod designation;
pub mod encode;
pub mod file_manager;
pub mod generic;
pub mod model;
pub mod profile;
pub mod scheduler;
pub mod schema;
pub mod show;
pub mod testing;
pub mod worker;
pub mod worker_manager;
pub mod ws;
pub mod ws_functions;

type Tx = UnboundedSender<Message>;
type PeerMap = HashMap<SocketAddr, (Option<i32>, Tx)>;

#[macro_use]
extern crate diesel;
//Every function that takes a Path can also take a PathBuf
//PathBuf output

//TODO: Allow an option to ensure (read/write) or check (read-only) that that path/directory/file exists on disk

pub fn os_string_to_string(os_string: &OsStr) -> String {
    match os_string.to_str() {
        Some(string) => string.to_string(),
        None => {
            error!("Failed to convert OsStr to &str");
            panic!();
        }
    }
}

pub fn pathbuf_with_suffix(path: &Path, suffix: String) -> PathBuf {
    get_parent_directory(path).join(format!(
        "{}{}.{}",
        get_file_stem(path),
        &suffix,
        get_extension(path),
    ))
}

pub fn get_file_stem(path: &Path) -> String {
    match path.file_stem() {
        Some(file_stem) => os_string_to_string(file_stem),
        None => {
            error!(
                "Couldn't get file stem from path: {}",
                pathbuf_to_string(path)
            );
            panic!();
        }
    }
}

pub fn get_file_name(path: &Path) -> String {
    match path.file_name() {
        Some(file_name) => os_string_to_string(file_name),
        None => {
            error!(
                "Couldn't get file name from path: {}",
                pathbuf_to_string(path)
            );
            panic!();
        }
    }
}

pub fn get_extension(path: &Path) -> String {
    match path.extension() {
        Some(extension) => os_string_to_string(extension),
        None => {
            error!(
                "Couldn't get file extension from path: {}",
                pathbuf_to_string(path)
            );
            panic!();
        }
    }
}

pub fn get_parent_directory(path: &Path) -> &Path {
    match path.parent() {
        Some(parent_path) => parent_path,
        None => panic!("Couldn't get parent from path: {}", pathbuf_to_string(path)),
    }
}

//Pathbuf/Path to String
pub fn pathbuf_to_string(path: &Path) -> String {
    match path.to_str() {
        Some(string) => string.to_string(),
        None => {
            error!("Failed to convert a path/pathbuf to a string");
            panic!();
        }
    }
}

//This function assumes a specific folder structure //tv_shows_network_share/tv_show_x/Season X/file
//tv_show_x is where the TV Show name information is pulled from
pub fn get_show_title_from_pathbuf(path: &Path) -> String {
    match path.parent() {
        Some(first_parent) => match first_parent.parent() {
            Some(second_parent) => match second_parent.file_name() {
                Some(directory_name) => {
                    return directory_name.to_string_lossy().to_string();
                }
                None => {
                    error!("Failed to get the directory name of the second parent of path. get_show_title_from_pathbuf()'s function header shows the required format.");
                    panic!();
                }
            },
            None => {
                error!("Failed to get second parent of path. get_show_title_from_pathbuf()'s function header shows the required format.");
                panic!();
            }
        },
        None => {
            error!("Failed to get first parent of path. get_show_title_from_pathbuf()'s function header shows the required format.");
            panic!();
        }
    }
}

pub fn create_file(test_file_path: &Path) -> Result<(), Error> {
    let mut file = File::create(pathbuf_to_string(test_file_path))?;
    file.write_all(b"Dummy unit testing file.")?;
    Ok(())
}

pub fn copy(source: &Path, destination: &Path) -> Result<u64, Error> {
    fs_copy(pathbuf_to_string(source), pathbuf_to_string(destination))
}

pub fn remove_file(path: &Path) -> Result<(), Error> {
    fs_remove_file(pathbuf_to_string(path))
}

pub fn ensure_path_exists(path: &Path) {
    if let Err(err) = fs::create_dir_all(path) {
        error!(
            "Failed to create \"tlm\" directory in temp directory. Error: {}",
            err
        );
        panic!();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebUIFileVersion {
    pub generic_uid: i32,
    pub id: i32,
    pub file_name: String,
}

impl WebUIFileVersion {
    pub fn from_file_version(file_version: &FileVersion) -> Self {
        Self {
            generic_uid: file_version.generic_uid,
            id: file_version.id,
            file_name: file_version.get_file_name(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageSource {
    Worker(WorkerMessage),
    WebUI(WebUIMessage),
}

impl MessageSource {
    pub fn expect_worker_message(message: Message) -> Option<WorkerMessage> {
        if message.is_text() {
            match message.into_text() {
                Ok(json) => {
                    let raw_message_source: Result<Self, serde_json::Error> = serde_json::from_str(&json);
                    match raw_message_source {
                        Ok(message_source) => {
                            match message_source {
                                MessageSource::Worker(worker_message) => {
                                    return Some(worker_message);
                                },
                                _ => return None,
                            }
                        },
                        Err(err) => {
                            error!("Failed converting json string to MessageSource, error output: {}", err);
                        },
                    }
                },
                Err(err) => {

                },
            }
        }
        None
    }

    pub fn from_message(message: Message) -> Option<Self> {
        if message.is_text() {
            match message.into_text() {
                Ok(json) => {
                    let raw_message_source: Result<Self, serde_json::Error> = serde_json::from_str(&json);
                    match raw_message_source {
                        Ok(message_source) => {
                            let message_source: MessageSource = message_source;
                            return Some(message_source);
                        },
                        Err(err) => {
                            error!("Failed converting json string to MessageSource, error output: {}", err);
                            panic!();
                        }
                    }
                },
                Err(err) => {
                    error!("Failed to convert message to text, dunno what went wrong at the time of writing this. Error: {}", err);
                    panic!();
                },
            }
        }
        None
    }

    pub fn to_message(&self) -> Message {
        match serde_json::to_string(&self) {
            Ok(json) => {
                Message::text(json)
            },
            Err(err) => {
                println!("Failed converting MessageSource to json string, error output: {}", err);
                panic!();
            },
        }
    }

    pub fn from_worker_message(worker_message: WorkerMessage) -> Self {
        Self::Worker(worker_message)
    }

    pub fn from_webui_message(webui_message: WebUIMessage) -> Self {
        Self::WebUI(webui_message)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequestType {
    AllFileVersions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WebUIMessage {
    //WebUI -> Server
    Request(RequestType),
    //EncodeGeneric(i32, i32, AddEncodeMode, EncodeProfile),

    //Server -> WebUI
    FileVersion(i32, i32, String),
    FileVersions(Vec<WebUIFileVersion>),
    //Generics(Vec<WebUIGeneric>),
}

impl WebUIMessage {
    pub fn to_message(self) -> Message {
        MessageSource::WebUI(self).to_message()
    }

    pub fn from_message(message: Message) -> Option<Self> {
        match MessageSource::from_message(message) {
            Some(MessageSource::WebUI(webui_message)) => {
                Some(webui_message)
            },
            _ => {
                warn!("Message received was not meant for the WebUI, that should already be known by this point.");
                None
            }
        }
    }
}
