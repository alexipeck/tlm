use super::generic::Generic;
use super::schema::{episode, generic, show};
use crate::profile::{convert_i32_to_container, convert_i32_to_resolution_standard, BasicProfile};

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
    pub resolution_standard: Option<i32>, //I want this to eventually be a string
    pub container: Option<i32>,           //I want this to eventually be a string
}

impl NewGeneric {
    pub fn new(full_path: String, designation: i32, profile: Option<BasicProfile>) -> Self {
        let mut temp = NewGeneric {
            full_path,
            designation,
            width: None,
            height: None,
            framerate: None,
            length_time: None,
            resolution_standard: None,
            container: None,
        };
        
        if let Some(profile) = profile {
            temp.width = Some(profile.width as i32);
            temp.height = Some(profile.height as i32);
            temp.framerate = Some(profile.framerate);
            temp.length_time = Some(profile.length_time);
            temp.resolution_standard = Some(profile.resolution_standard as i32);
            temp.container = Some(profile.container as i32);
        }
        temp
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
    pub resolution_standard: Option<i32>, //I want this to eventually be a string
    pub container: Option<i32>,           //I want this to eventually be a string
}

impl GenericModel {
    ///Create an in memory generic from a database one
    pub fn from_generic(generic: Generic) -> GenericModel {
        let mut temp = GenericModel {
            generic_uid: generic.generic_uid.unwrap() as i32,
            full_path: generic.get_full_path(),
            designation: generic.designation as i32,
            file_hash: generic.hash,
            fast_file_hash: generic.fast_hash,
            resolution_standard: None,
            container: None,
            width: None,
            height: None,
            framerate: None,
            length_time: None,
        };
        if generic.current_profile.is_some() {
            let current_profile = generic.current_profile.to_owned().unwrap();
            temp.width = Some(current_profile.width as i32);
            temp.height = Some(current_profile.height as i32);
            temp.framerate = Some(current_profile.framerate);
            temp.length_time = Some(current_profile.length_time);
            temp.resolution_standard = Some(current_profile.resolution_standard as i32);
            temp.container = Some(current_profile.container as i32);
        }

        temp
    }

    ///Construct a profile from database fields
    pub fn get_basic_profile(&self) -> Option<BasicProfile> {
        Some(BasicProfile {
            width: self.width? as u32,
            height: self.height? as u32,
            framerate: self.framerate?,
            length_time: self.length_time?,
            resolution_standard: convert_i32_to_resolution_standard(self.resolution_standard?),
            container: convert_i32_to_container(self.container?),
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