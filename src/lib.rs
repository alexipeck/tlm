#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    fs::{self, copy as fs_copy, remove_file as fs_remove_file, File},
    io::{Error, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use tracing::error;
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

pub fn pathbuf_with_suffix(path: &Path, suffix: String) -> PathBuf {
    get_parent_directory(path).join(format!(
        "{}{}.{}",
        pathbuf_to_string(&get_file_stem(path)),
        &suffix,
        pathbuf_to_string(&get_extension(path)),
    ))
}

pub fn get_file_stem(path: &Path) -> PathBuf {
    match path.file_stem() {
        Some(file_stem) => PathBuf::from(file_stem),
        None => {
            error!(
                "Couldn't get file stem from path: {}",
                pathbuf_to_string(path)
            );
            panic!();
        }
    }
}

pub fn get_file_name(path: &Path) -> PathBuf {
    match path.file_name() {
        Some(file_name) => PathBuf::from(file_name),
        None => {
            error!(
                "Couldn't get file name from path: {}",
                pathbuf_to_string(path)
            );
            panic!();
        }
    }
}

pub fn get_extension(path: &Path) -> PathBuf {
    match path.extension() {
        Some(extension) => PathBuf::from(extension),
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

///Pathbuf/Path to String
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