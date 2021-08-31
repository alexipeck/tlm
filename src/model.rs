use super::generic::Content;
use super::schema::{content, episode, show};

#[derive(Insertable)]
#[table_name = "content"]
pub struct NewContent {
    pub full_path: String,
    pub designation: i32,
}

#[derive(Queryable, AsChangeset, Identifiable)]
#[table_name = "content"]
pub struct ContentModel {
    pub id: i32,
    pub full_path: String,
    pub designation: i32,
    pub file_hash: Option<String>,
}

impl ContentModel {
    pub fn from_content(c: Content) -> ContentModel {
        return ContentModel {
            id: c.content_uid.unwrap() as i32,
            full_path: c.get_full_path(),
            designation: c.designation as i32,
            file_hash: c.hash,
        };
    }
}

#[derive(Insertable)]
#[table_name = "episode"]
pub struct NewEpisode {
    pub content_uid: i32,
    pub show_uid: i32,
    pub episode_title: String,
    pub season_number: i32,
    pub episode_number: i32,
}

#[derive(Queryable)]
pub struct EpisodeModel {
    pub content_uid: i32,
    pub show_uid: i32,
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
    pub title: String,
}

#[derive(Queryable)]
pub struct ShowModel {
    pub show_uid: i32,
    pub title: String,
}
