pub mod config;
pub mod content;
pub mod database;
pub mod designation;
pub mod error_handling;
pub mod job;
pub mod manager;
pub mod print;
pub mod queue;
pub mod task;
pub mod timer;
pub mod tv;
pub mod utility;
pub mod schema;
pub mod model;

#[macro_use]
extern crate diesel;

use content::Content;
use std::{collections::HashSet, fs, path::PathBuf, time::Instant};
use tv::show::Show;
use twox_hash::xxh3;
use utility::Utility;



use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use model::*;

pub fn establish_connection() -> PgConnection {

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_content<'a>(conn: &PgConnection, full_path: String, designation: i32) -> ContentModel {
    use schema::content;
    
    let new_content = NewContent {
        full_path: full_path,
        designation: designation
    };

    diesel::insert_into(content::table)
        .values(&new_content)
        .get_result(conn)
        .expect("Error saving new content")
}

pub fn create_episode<'a>(conn: &PgConnection, content_uid: i32, show_uid: i32, episode_title: String, season_number: i32, episode_number: i32) -> EpisodeModel {
    use schema::episode;
    
    let new_episode = NewEpisode {
        content_uid: content_uid,
        show_uid: show_uid,
        episode_title: episode_title,
        season_number: season_number,
        episode_number: episode_number
    };

    diesel::insert_into(episode::table)
        .values(&new_episode)
        .get_result(conn)
        .expect("Error saving new episode")
}

pub fn load_from_database(utility: Utility) -> (Vec<Content>, Vec<Show>, HashSet<PathBuf>) {
    let utility = utility.clone_and_add_location("load_from_database");

    let mut working_shows: Vec<Show> = Show::get_all_shows(utility.clone());

    let working_content = Content::get_all_contents(&mut working_shows, utility.clone());

    let existing_files_hashset: HashSet<PathBuf> =
        Content::get_all_filenames_as_hashset_from_contents(
            working_content.clone(),
            utility.clone(),
        );

    return (working_content, working_shows, existing_files_hashset);
}

pub fn get_show_title_from_pathbuf(pathbuf: &PathBuf) -> String {
    return pathbuf
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
}

pub fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
}
