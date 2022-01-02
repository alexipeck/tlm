#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    fs::{copy, remove_file, File, self},
    io::{Error, Write, self},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error};
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
pub mod unit_tests;
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
    pathbuf_get_parent(path).join(format!(
        "{}{}.{}",
        pathbuf_to_string(&pathbuf_file_stem(path)),
        &suffix,
        pathbuf_to_string(&pathbuf_extension(path)),
    ))
}

pub fn pathbuf_file_stem(path: &Path) -> PathBuf {
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

pub fn pathbuf_file_name(path: &Path) -> PathBuf {
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

pub fn pathbuf_extension(path: &Path) -> PathBuf {
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

//Path output
pub fn pathbuf_get_parent(path: &Path) -> &Path {
    match path.parent() {
        Some(parent_path) => parent_path,
        None => panic!("Couldn't get parent from path: {}", pathbuf_to_string(path)),
    }
}

//String output
pub fn pathbuf_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

pub fn get_show_title_from_pathbuf(path: &Path) -> String {
    path.parent()
        .unwrap()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub fn pathbuf_create_file(test_file_path: &Path) -> Result<(), Error> {
    let mut file = File::create(pathbuf_to_string(test_file_path))?;
    file.write_all(b"Dummy unit testing file.")?;
    Ok(())
}

pub fn pathbuf_copy(source: &Path, destination: &Path) -> Result<u64, Error> {
    copy(pathbuf_to_string(source), pathbuf_to_string(destination))
}

pub fn pathbuf_remove_file(path: &Path) -> Result<(), Error> {
    remove_file(pathbuf_to_string(path))
}

pub fn ensure_path_exists(path: &Path) {
    if let Err(err) = fs::create_dir_all(path) {
        error!("Failed to create \"tlm\" directory in temp directory. Error: {}", err);
        panic!();
    }
}