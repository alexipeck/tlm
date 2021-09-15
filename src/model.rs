use super::generic::Generic;
use super::schema::{episode, generic, show};
use crate::profile::Profile;

///Struct for inserting into the database
#[derive(Insertable)]
#[table_name = "generic"]
pub struct NewGeneric {
    pub full_path: String,
    pub designation: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
}

///Helper function to set all database fields to none if the profile contains data
fn profile_is_some_split(
    profile: Option<Profile>,
) -> (Option<i32>, Option<i32>, Option<f64>, Option<f64>) {
    match profile {
        Some(profile) => (
            Some(profile.width as i32),
            Some(profile.height as i32),
            Some(profile.framerate),
            Some(profile.length_time),
        ),
        None => (None, None, None, None),
    }
}

impl NewGeneric {
    pub fn new(full_path: String, designation: i32, profile: Option<Profile>) -> Self {
        let temp_profile = profile_is_some_split(profile);

        NewGeneric {
            full_path,
            designation,
            width: temp_profile.0,
            height: temp_profile.1,
            framerate: temp_profile.2,
            length_time: temp_profile.3,
        }
    }
}

///Data structure to modify or select an existing Generic in the database
#[derive(Queryable, AsChangeset, Identifiable)]
#[primary_key(generic_uid)]
#[table_name = "generic"]
pub struct GenericModel {
    pub generic_uid: i32,
    pub full_path: String,
    pub designation: i32,
    pub file_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
    pub fast_file_hash: Option<String>,
}

impl GenericModel {
    ///Create an in memory generic from a database one
    pub fn from_generic(generic: Generic) -> GenericModel {
        if generic.profile.is_some() {
            GenericModel {
                generic_uid: generic.generic_uid.unwrap() as i32,
                full_path: generic.get_full_path(),
                designation: generic.designation as i32,
                file_hash: generic.hash,
                fast_file_hash: generic.fast_hash,
                width: Some(generic.profile.to_owned().unwrap().width as i32),
                height: Some(generic.profile.to_owned().unwrap().height as i32),
                framerate: Some(generic.profile.to_owned().unwrap().framerate),
                length_time: Some(generic.profile.unwrap().length_time),
            }
        } else {
            GenericModel {
                generic_uid: generic.generic_uid.unwrap() as i32,
                full_path: generic.get_full_path(),
                designation: generic.designation as i32,
                file_hash: generic.hash,
                fast_file_hash: generic.fast_hash,
                width: None,
                height: None,
                framerate: None,
                length_time: None,
            }
        }
    }

    ///Construct a profile from database fields
    pub fn get_profile(&self) -> Option<Profile> {
        Some(Profile {
            width: self.width? as u32,
            height: self.height? as u32,
            framerate: self.framerate?,
            length_time: self.length_time?,
        })
    }
}

///Structure for inserting an episode into the database
#[derive(Insertable)]
#[table_name = "episode"]
pub struct NewEpisode {
    pub generic_uid: i32,
    pub show_uid: i32,
    pub episode_title: String,
    pub season_number: i32,
    pub episode_number: i32,
}

impl NewEpisode {
    pub fn new(
        generic_uid: usize,
        show_uid: usize,
        episode_title: String,
        season_number: usize,
        episode_number: usize,
    ) -> Self {
        NewEpisode {
            generic_uid: generic_uid as i32,
            show_uid: show_uid as i32,
            episode_title,
            season_number: season_number as i32,
            episode_number: episode_number as i32,
        }
    }
}

///Structure to select Episodes from the database
#[derive(Queryable)]
pub struct EpisodeModel {
    pub generic_uid: i32,
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

///Struct to insert shows into the database
#[derive(Insertable)]
#[table_name = "show"]
pub struct NewShow {
    pub show_title: String,
}

///Struct to select shows from the database
#[derive(Queryable)]
pub struct ShowModel {
    pub show_uid: i32,
    pub show_title: String,
}
