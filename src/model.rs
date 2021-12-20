use super::generic::Generic;
use super::schema::{episode, file_version, generic, show, worker};
use crate::generic::FileVersion;
use crate::worker::Worker;

//Workers
#[derive(Insertable)]
#[table_name = "worker"]
pub struct NewWorker {
    pub worker_ip_address: String,
}

impl NewWorker {
    pub fn new(worker_ip_address: String) -> Self {
        Self { worker_ip_address }
    }

    pub fn from_worker(worker: Worker) -> Self {
        let ip = worker.worker_ip_address.to_string();
        NewWorker {
            worker_ip_address: ip,
        }
    }
}

pub fn from_worker(worker: Worker) -> NewWorker {
    NewWorker::new(worker.worker_ip_address.to_string())
}

#[derive(Queryable, AsChangeset, Identifiable)]
#[primary_key(id)]
#[table_name = "worker"]
pub struct WorkerModel {
    pub id: i32,
    pub worker_ip_address: String,
}

//Generic
///Struct for inserting into the database
#[derive(Insertable)]
#[table_name = "generic"]
pub struct NewGeneric {
    pub designation: i32,
}

impl NewGeneric {
    pub fn new(designation: i32) -> Self {
        Self { designation }
    }
}

///Data structure to modify or select an existing Generic in the database
#[derive(Queryable, AsChangeset, Identifiable)]
#[primary_key(generic_uid)]
#[table_name = "generic"]
pub struct GenericModel {
    pub generic_uid: i32,
    pub designation: i32,
}

impl GenericModel {
    pub fn from_generic(generic: Generic) -> Self {
        Self {
            generic_uid: generic.get_generic_uid(),
            designation: generic.designation as i32,
        }
    }
}

//FileVersion
#[derive(Insertable)]
#[table_name = "file_version"]
pub struct NewFileVersion {
    generic_uid: i32,
    full_path: String,
    master_file: bool,
    file_hash: Option<String>,
    fast_file_hash: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    framerate: Option<f64>,
    length_time: Option<f64>,
    resolution_standard: Option<i32>,
    container: Option<i32>,
}

impl NewFileVersion {
    pub fn new(generic_uid: i32, full_path: String, master_file: bool) -> Self {
        Self {
            generic_uid,
            full_path,
            master_file,
            file_hash: None,
            fast_file_hash: None,
            width: None,
            height: None,
            framerate: None,
            length_time: None,
            resolution_standard: None,
            container: None,
        }
    }
}

#[derive(Queryable, AsChangeset, Identifiable, Clone)]
#[primary_key(id)]
#[table_name = "file_version"]
pub struct FileVersionModel {
    pub id: i32,
    pub generic_uid: i32,
    pub full_path: String,
    pub master_file: bool,
    pub file_hash: Option<String>,
    pub fast_file_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
    pub resolution_standard: Option<i32>,
    pub container: Option<i32>,
}

impl FileVersionModel {
    pub fn from_file_version(file_version: FileVersion) -> Self {
        let mut resolution_standard: Option<i32> = None;
        if file_version.profile.resolution_standard.is_some() {
            resolution_standard = Some(file_version.profile.resolution_standard.unwrap() as i32);
        }

        let mut container: Option<i32> = None;
        if file_version.profile.container.is_some() {
            container = Some(file_version.profile.container.unwrap() as i32);
        }

        Self {
            id: file_version.id,
            generic_uid: file_version.generic_uid,
            full_path: file_version.get_full_path(),
            file_hash: file_version.hash,
            master_file: file_version.master_file,
            fast_file_hash: file_version.fast_hash,
            width: file_version.profile.width,
            height: file_version.profile.height,
            framerate: file_version.profile.framerate,
            length_time: file_version.profile.length_time,
            resolution_standard,
            container,
        }
    }
}

//Episode
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
        generic_uid: i32,
        show_uid: i32,
        episode_title: String,
        season_number: i32,
        episode_number: i32,
    ) -> Self {
        Self {
            generic_uid,
            show_uid,
            episode_title,
            season_number,
            episode_number,
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

//Show
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
