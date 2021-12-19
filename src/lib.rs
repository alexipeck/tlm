#![doc = include_str!("../README.md")]

use std::path::PathBuf;
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

pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
    return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
}
