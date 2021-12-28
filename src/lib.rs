#![doc = include_str!("../README.md")]

use std::path::{Path, PathBuf};
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
    pathbuf_get_parent(path).join(format!(
        "{}{}.{}",
        pathbuf_file_stem_to_string(path),
        &suffix,
        pathbuf_extension_to_string(path),
    ))
}

pub fn pathbuf_file_stem(path: &Path) -> PathBuf {
    match path.file_stem() {
        Some(file_stem) => PathBuf::from(file_stem),
        None => panic!("Couldn't get file stem"),
    }
}

//Path output
pub fn pathbuf_get_parent(path: &Path) -> &Path {
    match path.parent() {
        Some(parent_path) => parent_path,
        None => panic!("Couldn't get parent"),
    }
}

//String output
pub fn pathbuf_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

pub fn pathbuf_to_string_with_suffix(path: &Path, suffix: String) -> String {
    pathbuf_to_string(&pathbuf_get_parent(path).join(format!(
        "{}{}.{}",
        pathbuf_file_stem_to_string(path),
        &suffix,
        pathbuf_extension_to_string(path),
    )))
}

pub fn pathbuf_file_name_to_string(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}

pub fn pathbuf_extension_to_string(path: &Path) -> String {
    path.extension().unwrap().to_str().unwrap().to_string()
}

pub fn pathbuf_file_stem_to_string(path: &Path) -> String {
    path.file_stem().unwrap().to_str().unwrap().to_string()
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
