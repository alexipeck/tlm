use super::content::Content;
use super::schema::{content, episode, show};
use crate::profile::Profile;

#[derive(Insertable)]
#[table_name = "content"]
pub struct NewContent {
    pub full_path: String,
    pub designation: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
}

#[derive(Queryable, AsChangeset, Identifiable)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "content"]
pub struct ContentModel {
    pub id: i32,
    pub full_path: String,
    pub designation: i32,
    pub file_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
}

impl ContentModel {
    pub fn get_profile(&self) -> Option<Profile> {
        Some(Profile {
            width: self.width? as u32,
            height: self.height? as u32,
            framerate: self.framerate?,
            length_time: self.length_time?,
        })
    }
    pub fn from_content(c: Content) -> ContentModel {
        if c.profile.is_some() {
            ContentModel {
                id: c.content_uid.unwrap() as i32,
                full_path: c.get_full_path(),
                designation: c.designation as i32,
                file_hash: c.hash,
                width: Some(c.profile.to_owned().unwrap().width as i32),
                height: Some(c.profile.to_owned().unwrap().height as i32),
                framerate: Some(c.profile.to_owned().unwrap().framerate),
                length_time: Some(c.profile.unwrap().length_time),
            }
        } else {
            ContentModel {
                id: c.content_uid.unwrap() as i32,
                full_path: c.get_full_path(),
                designation: c.designation as i32,
                file_hash: c.hash,
                width: None,
                height: None,
                framerate: None,
                length_time: None,
            }
        }
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
