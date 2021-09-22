#![doc = include_str!("../README.md")]
pub mod config;
pub mod database;
pub mod designation;
pub mod generic;
pub mod manager;
pub mod model;
pub mod profile;
pub mod scheduler;
pub mod schema;
pub mod show;
pub mod ws;

#[macro_use]
extern crate diesel;
