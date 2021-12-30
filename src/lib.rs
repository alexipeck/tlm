#![doc = include_str!("../README.md")]

use std::{path::{Path, PathBuf}, fs::{copy, remove_file}, io::Error};

use tracing::{error, debug};
pub mod config;
pub mod database;
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

#[macro_use]
extern crate diesel;
//Every function that takes a Path can also take a PathBuf
//PathBuf output
pub fn pathbuf_with_suffix(path: &Path, suffix: String) -> PathBuf {
    debug!("pathbuf_with_suffix: {}", pathbuf_to_string(path));
    debug!("pathbuf_with_suffix: {}", pathbuf_to_string(&pathbuf_extension(path)));

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
            error!("Couldn't get file stem from path: {}", pathbuf_to_string(path));
            panic!();
        },
    }
}

pub fn pathbuf_file_name(path: &Path) -> PathBuf {
    match path.file_name() {
        Some(file_name) => {
            PathBuf::from(file_name)
        },
        None => {
            error!("Couldn't get file name from path: {}", pathbuf_to_string(path));
            panic!();
        },
    }
}

pub fn pathbuf_extension(path: &Path) -> PathBuf {
    match path.extension() {
        Some(extension) => {
            PathBuf::from(extension)
        },
        None => {
            error!("Couldn't get file extension from path: {}", pathbuf_to_string(path));
            panic!();
        },
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

pub fn pathbuf_copy(source: &Path, destination: &Path) -> Result<u64, Error> {
    copy(pathbuf_to_string(source), pathbuf_to_string(destination))
}


pub fn pathbuf_remove_file(path: &Path) -> Result<(), Error> {
    remove_file(pathbuf_to_string(path))
}