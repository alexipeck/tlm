//!Datatype and associated function for handling Generic video files as well as the generic
//!information used by all other video file types
use std::io::prelude::*;
use std::{fs::File, path::PathBuf};

use std::fmt;
use std::hash::Hasher;

use crate::profile::Profile;
use crate::worker_manager::Encode;
use crate::{
    designation::{from_i32, Designation},
    model::*,
};
use crate::{pathbuf_file_name_to_string, pathbuf_to_string, pathbuf_with_suffix};
use rand::Rng;
use tracing::{error, warn, debug};

#[derive(Clone, Debug)]
pub struct FileVersion {
    pub id: i32,
    pub generic_uid: i32,
    pub full_path: PathBuf,
    pub master_file: bool,
    pub hash: Option<String>,
    pub fast_hash: Option<String>,
    pub profile: Profile,
}

impl FileVersion {
    pub fn from_file_version_model(file_version_model: FileVersionModel) -> Self {
        let profile = Profile::new(
            file_version_model.width,
            file_version_model.height,
            file_version_model.framerate,
            file_version_model.length_time,
            file_version_model.resolution_standard,
            file_version_model.container,
        );

        Self {
            id: file_version_model.id,
            generic_uid: file_version_model.generic_uid,
            full_path: PathBuf::from(file_version_model.full_path),
            master_file: file_version_model.master_file,
            hash: file_version_model.file_hash,
            fast_hash: file_version_model.fast_file_hash,
            profile,
        }
    }

    //Destructive operation, will overwrite previous values
    pub fn generate_profile(&mut self) {
        if let Some(profile) = Profile::from_file(&self.full_path) {
            self.profile = profile;
        } else {
            panic!(
                "Failed to generate profile for generic_uid: {} and file_version_id: {}",
                self.generic_uid, self.id
            );
        }
    }

    ///Hash the file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        self.hash = Some(sea_hash(self.full_path.clone()));
    }

    ///Returns true if hashes match, false if not
    pub fn verify_hash(&mut self, path: PathBuf) -> bool {
        if self.hash.is_some() {
            return self.hash.as_ref().unwrap().as_str() == sea_hash(path).as_str();
        } else {
            warn!("Fast hash verification was run on a file without a hash. Continuing with the assumption that this is intentional");
            true
        }
    }

    ///Hash the first 32MB of the file with seahash so we can quickly know
    ///if a file is likely to have changed or is likely to be the same as
    ///an existing file.
    ///
    ///For example if we backup all of tlm's information and all files get
    ///renamed to something that doesn't make sense we can quickly search for
    ///files that tlm knows about to restore by calculating the fast hash and
    ///then calculating full hashes of matching hashes to save time
    pub fn fast_hash(&mut self) {
        self.fast_hash = Some(sea_fast_hash(self.full_path.clone()));
    }

    ///Returns true if hashes match, false if not
    pub fn verify_fast_hash(&mut self, path: PathBuf) -> bool {
        if self.fast_hash.is_some() {
            return self.fast_hash.as_ref().unwrap().as_str() == sea_fast_hash(path).as_str();
        } else {
            warn!("Fast hash verification was run on a file without a hash. Continuing with the assumption that this is intentional");
            true
        }
    }

    pub fn generate_encode(&self) -> Encode {
        Encode::new(
            self.generic_uid,
            self.full_path.clone(),
            self.generate_target_path(),
            self.generate_encode_string(),
        )
    }

    ///Returns a vector of ffmpeg arguments for later execution
    ///This has no options currently
    pub fn generate_encode_string(&self) -> Vec<String> {
        let mut encode_string = vec!["-i".to_string(), self.get_full_path()];

        //Video
        encode_string.push("-c:v".to_string());
        encode_string.push("libx265".to_string());
        encode_string.push("-crf".to_string());
        encode_string.push("25".to_string());
        encode_string.push("-preset".to_string());
        encode_string.push("slower".to_string());
        encode_string.push("-profile:v".to_string());
        encode_string.push("main".to_string());

        //Audio
        encode_string.push("-c:a".to_string());
        encode_string.push("aac".to_string());
        encode_string.push("-q:a".to_string());
        encode_string.push("224k".to_string());

        encode_string.push("-y".to_string());
        encode_string.push(pathbuf_to_string(&self.generate_target_path()));
        encode_string
    }

    ///Appends a fixed string to differentiate rendered files from original before overwrite
    /// I doubt this will stay as I think a temp directory would be more appropriate.
    /// This function returns that as a string for the ffmpeg arguments
    pub fn generate_target_path(&self) -> PathBuf {
        self.get_full_path_with_suffix_as_pathbuf(format!(
            "_temp_test_encode{}",
            rand::thread_rng().gen::<i32>()
        ))
    }

    pub fn get_filename(&self) -> String {
        pathbuf_file_name_to_string(&self.full_path)
    }

    fn get_full_path_with_suffix_as_pathbuf(&self, suffix: String) -> PathBuf {
        pathbuf_with_suffix(&self.full_path, suffix)
    }

    pub fn get_full_path(&self) -> String {
        pathbuf_to_string(&self.full_path)
    }
}

///Struct containing data that is shared by all file types
///can also refer to only a generic media file
#[derive(Clone, Debug)]
pub struct Generic {
    pub generic_uid: Option<i32>,
    pub designation: Designation,
    pub file_versions: Vec<FileVersion>,
}

impl Generic {
    pub fn default() -> Self {
        Self {
            generic_uid: None,
            designation: Designation::Generic,
            file_versions: Vec::new(),
        }
    }

    pub fn insert_file_version(&mut self, file_version: FileVersion) {
        self.file_versions.push(file_version)
    }

    pub fn hash_file_versions(&mut self, file_version_count: usize, generics_iter_progress: usize) {
        let length: usize = self.file_versions.len();
        for (i, file_version) in self.file_versions.iter_mut().enumerate() {
            if file_version.hash.is_none() {
                file_version.hash();
            }
            if file_version.fast_hash.is_none() {
                file_version.fast_hash();
            }
            debug!("Hashed[[{} of {}][{:2} of {:2}]]: {}", i + 1, length, generics_iter_progress, file_version_count, pathbuf_to_string(&file_version.full_path));
            
        }
    }

    pub fn get_all_full_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for file_version in &self.file_versions {
            paths.push(file_version.full_path.clone());
        }
        paths
    }

    pub fn get_file_version_by_id(&self, file_version_id: i32) -> Option<FileVersion> {
        for file_version in &self.file_versions {
            if file_version.id == file_version_id {
                return Some(file_version.clone());
            }
        }
        None
    }

    pub fn has_hashing_work(&self) -> bool {
        for file_version in &self.file_versions {
            if file_version.hash.is_none() || file_version.fast_hash.is_none() {
                return true;
            }
        }
        false
    }

    pub fn get_master_full_path(&self) -> String {
        pathbuf_to_string(&self.file_versions[0].full_path)
    }

    ///Create a new generic from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_generic_model(generic_model: GenericModel) -> Self {
        Self {
            generic_uid: Some(generic_model.generic_uid),
            designation: from_i32(generic_model.designation),
            file_versions: Vec::new(),
        }
    }

    pub fn get_generic_uid(&self) -> i32 {
        if self.generic_uid.is_some() {
            self.generic_uid.unwrap()
        } else {
            error!("get_generic_uid was called on a generic that hasn't been inserted into the db yet or hasn't been assigned a generic_uid from the database correctly");
            panic!();
        }
    }
}
///Hash the file with seahash for data integrity purposes so we
/// know if a file has been replaced and may need to be reprocessed
pub fn sea_hash(full_path: PathBuf) -> String {
    let mut buffer = Box::new(vec![0; 4096]);
    let mut hasher = seahash::SeaHasher::new();
    let mut file = File::open(pathbuf_to_string(&full_path)).unwrap_or_else(|err| {
        error!("Error opening file for hashing. Err: {}", err);
        panic!();
    });
    while file.read(&mut buffer).unwrap() != 0 {
        hasher.write(&buffer);
    }
    hasher.finish().to_string()
}

///Hash the first 32MB of the file with seahash so we can quickly know
///if a file is likely to have changed or is likely to be the same as
///an existing file.
///
///For example if we backup all of tlm's information and all files get
///renamed to something that doesn't make sense we can quickly search for
///files that tlm knows about to restore by calculating the fast hash and
///then calculating full hashes of matching hashes to save time
pub fn sea_fast_hash(full_path: PathBuf) -> String {
    let mut buffer = Box::new(vec![0; 4096]);
    let mut hasher = seahash::SeaHasher::new();
    let mut file = File::open(pathbuf_to_string(&full_path)).unwrap_or_else(|err| {
        error!("Error opening file for hashing. Err: {}", err);
        panic!();
    });
    for _ in 0..8192 {
        if file.read(&mut buffer).unwrap() != 0 {
            hasher.write(&buffer);
        } else {
            break;
        }
    }
    hasher.finish().to_string()
}

impl fmt::Display for Generic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.file_versions.is_empty() {
            panic!("The server was told to print a generic that has no actual files");
        }
        let mut temp: String = String::new();
        for file_version in &self.file_versions {
            temp.push_str(&format!(
                "[designation:'{}'][full_path:'{}']",
                self.designation as i32,
                file_version.get_full_path(),
            ));
        }
        write!(f, "{}", temp)
    }
}
