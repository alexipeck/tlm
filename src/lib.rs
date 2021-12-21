#![doc = include_str!("../README.md")]

use std::path::Path;
pub mod config;
pub mod database;
pub mod designation;
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

//Path output
pub fn pathbuf_get_parent(pathbuf: &Path) -> &Path {
    pathbuf.parent().unwrap()
}

//String output
pub fn pathbuf_to_string(pathbuf: &Path) -> String {
    pathbuf.to_str().unwrap().to_string()
}

pub fn pathbuf_to_string_with_suffix(pathbuf: &Path, suffix: String) -> String {
    pathbuf_to_string(&pathbuf_get_parent(pathbuf).join(format!(
        "{}{}.{}",
        pathbuf_file_stem_to_string(pathbuf),
        &suffix,
        pathbuf_extension_to_string(pathbuf),
    )))
}

pub fn path_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_string()
}

pub fn pathbuf_file_name_to_string(pathbuf: &Path) -> String {
    pathbuf.file_name().unwrap().to_str().unwrap().to_string()
}

pub fn pathbuf_extension_to_string(pathbuf: &Path) -> String {
    pathbuf.extension().unwrap().to_str().unwrap().to_string()
}

pub fn pathbuf_file_stem_to_string(pathbuf: &Path) -> String {
    pathbuf.file_stem().unwrap().to_str().unwrap().to_string()
}

pub fn get_show_title_from_pathbuf(pathbuf: &Path) -> String {
    pathbuf.parent().unwrap().parent().unwrap().file_name().unwrap().to_string_lossy().to_string()
}
