#![doc = include_str!("../README.md")]
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
pub mod worker_manager;
pub mod ws;

#[macro_use]
extern crate diesel;
