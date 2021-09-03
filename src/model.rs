use super::generic::Generic;
use super::schema::{episode, generic, show};

#[derive(Insertable)]
#[table_name = "generic"]
pub struct NewGeneric {
    pub full_path: String,
    pub designation: i32,
}

#[derive(Queryable, AsChangeset, Identifiable)]
#[table_name = "generic"]
pub struct GenericModel {
    pub id: i32,
    pub full_path: String,
    pub designation: i32,
    pub file_hash: Option<String>,
}

impl GenericModel {
    pub fn from_generic(c: Generic) -> GenericModel {
        return GenericModel {
            id: c.generic_uid.unwrap() as i32,
            full_path: c.get_full_path(),
            designation: c.designation as i32,
            file_hash: c.hash,
        };
    }
}

#[derive(Insertable)]
#[table_name = "episode"]
pub struct NewEpisode {
    pub generic_uid: i32,
    pub show_uid: i32,
    pub show_title: String,
    pub episode_title: String,
    pub season_number: i32,
    pub episode_number: i32,
}

impl NewEpisode {
    pub fn new(
        generic_uid: usize,
        show_uid: usize,
        show_title: String,
        episode_title: String,
        season_number: usize,
        episode_number: usize,
    ) -> Self {
        return NewEpisode {
            generic_uid: generic_uid as i32,
            show_uid: show_uid as i32,
            show_title: show_title,
            episode_title: episode_title,
            season_number: season_number as i32,
            episode_number: episode_number as i32,
        };
    }
}

#[derive(Queryable)]
pub struct EpisodeModel {
    pub generic_uid: i32,
    pub show_uid: i32,
    pub show_title: String,
    pub episode_title: String,
    pub season_number: i32,
    pub episode_number: i32,
}

#[derive(Queryable)]
pub struct JobQueueModel {
    pub job_uid: i32,
    pub source_path: String,
    pub encode_path: String,
    pub cache_directory: String,
    pub encode_string: String,
    pub status_underway: bool,
    pub status_completed: bool,
    pub worker_uid: i32,
    pub worker_string_id: String,
}

#[derive(Queryable)]
pub struct JobTaskQueueModel {
    pub id: i32,
    pub job_uid: i32,
    pub task_id: i32,
}

#[derive(Insertable)]
#[table_name = "show"]
pub struct NewShow {
    pub show_title: String,
}

#[derive(Queryable)]
pub struct ShowModel {
    pub show_uid: i32,
    pub show_title: String,
}
