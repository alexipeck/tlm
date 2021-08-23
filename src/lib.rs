pub mod config;
pub mod content;
pub mod designation;
pub mod error_handling;
pub mod job;
pub mod manager;
pub mod model;
pub mod print;
pub mod queue;
pub mod schema;
pub mod task;
pub mod timer;
pub mod tv;
pub mod utility;

#[macro_use]
extern crate diesel;

use content::Content;
use std::{collections::HashSet, fs, path::PathBuf, time::Instant};
use tv::Show;
use twox_hash::xxh3;
use utility::Utility;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use model::*;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_content<'a>(
    conn: &PgConnection,
    full_path: String,
    designation: i32,
) -> ContentModel {
    use schema::content;

    let new_content = NewContent {
        full_path: full_path,
        designation: designation,
    };

    diesel::insert_into(content::table)
        .values(&new_content)
        .get_result(conn)
        .expect("Error saving new content")
}

pub fn create_show<'a>(conn: &PgConnection, title: String) -> ShowModel {
    use schema::show;

    let new_show = NewShow { title: title };

    diesel::insert_into(show::table)
        .values(&new_show)
        .get_result(conn)
        .expect("Error saving new show")
}

pub fn create_episode<'a>(
    conn: &PgConnection,
    content_uid: i32,
    show_uid: i32,
    episode_title: String,
    season_number: i32,
    episode_number: i32,
) -> EpisodeModel {
    use schema::episode;

    let new_episode = NewEpisode {
        content_uid: content_uid,
        show_uid: show_uid,
        episode_title: episode_title,
        season_number: season_number,
        episode_number: episode_number,
    };

    diesel::insert_into(episode::table)
        .values(&new_episode)
        .get_result(conn)
        .expect("Error saving new episode")
}

pub fn load_from_database(utility: Utility) -> (Vec<Content>, Vec<Show>, HashSet<PathBuf>) {
    let mut utility = utility.clone_add_location_start_timing("load_from_database", 0);

    let mut working_shows: Vec<Show> = Show::get_all_shows(utility.clone());
    let working_content = Content::get_all_contents(&mut working_shows, utility.clone());
    let existing_files_hashset: HashSet<PathBuf> =
        Content::get_all_filenames_as_hashset_from_content(
            working_content.clone(),
            utility.clone(),
        );

    utility.print_function_timer();
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
