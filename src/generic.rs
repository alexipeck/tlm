//!Datatype and associated function for handling Generic video files as well as the generic
//!information used by all other video file types

use crate::database::create_file_version;
use {
    crate::{
        database::update_file_version,
        designation::{from_i32, Designation},
        get_file_name,
        model::*,
        pathbuf_to_string,
        profile::{Container, Profile, ResolutionStandard},
    },
    diesel::PgConnection,
    std::{fmt, fs::File, hash::Hasher, io::prelude::*, path::Path, path::PathBuf},
    tracing::{error, warn},
};

#[derive(Clone, Debug)]
pub struct FileVersion {
    pub id: i32,
    pub generic_uid: i32,
    pub full_path: PathBuf,
    pub master_file: bool,
    pub hash: Option<String>,
    pub fast_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub framerate: Option<f64>,
    pub length_time: Option<f64>,
    pub resolution_standard: Option<ResolutionStandard>,
    pub container: Option<Container>,
}

impl FileVersion {
    pub fn new(generic_uid: i32, full_path: &Path, master_file: bool) -> Self {
        Self::from_model(create_file_version(NewFileVersion::new(generic_uid, pathbuf_to_string(full_path), master_file)))
    }

    pub fn from_model(model: FileVersionModel) -> Self {
        Self {
            id: model.id,
            generic_uid: model.generic_uid,
            full_path: PathBuf::from(model.full_path),
            master_file: model.master_file,
            hash: model.file_hash,
            fast_hash: model.fast_file_hash,
            width: model.width,
            height: model.height,
            framerate: model.framerate,
            length_time: model.length_time,
            resolution_standard: ResolutionStandard::from_wrapped(
                model.resolution_standard,
            ),
            container: Container::from_wrapped(model.container),
        }
    }

    pub fn from_model_ref(model_ref: &FileVersionModel) -> Self {
        Self {
            id: model_ref.id,
            generic_uid: model_ref.generic_uid,
            full_path: PathBuf::from(model_ref.full_path.clone()),
            master_file: model_ref.master_file,
            hash: model_ref.file_hash.clone(),
            fast_hash: model_ref.fast_file_hash.clone(),
            width: model_ref.width,
            height: model_ref.height,
            framerate: model_ref.framerate,
            length_time: model_ref.length_time,
            resolution_standard: ResolutionStandard::from_wrapped(
                model_ref.resolution_standard,
            ),
            container: Container::from_wrapped(model_ref.container),
        }
    }

    pub fn update_file_version(&self, connection: &PgConnection) {
        update_file_version(self, connection);
    }

    pub fn profile_is_none(&self) -> bool {
        self.width.is_none()
            || self.height.is_none()
            || self.framerate.is_none()
            || self.length_time.is_none()
            || self.resolution_standard.is_none()
            || self.container.is_none()
    }

    //TODO: Add reporting
    //Destructive operation, will overwrite previous values
    pub fn generate_profile_if_none(&mut self, connection: &PgConnection) {
        if self.profile_is_none() {
            self.generate_profile(connection);
        }
    }

    //Destructive operation, will overwrite previous values
    pub fn generate_profile(&mut self, connection: &PgConnection) {
        if let Some(profile) = Profile::from_file(&self.full_path) {
            self.width = profile.width;
            self.height = profile.height;
            self.framerate = profile.framerate;
            self.length_time = profile.length_time;
            self.resolution_standard = profile.resolution_standard;
            self.container = profile.container;
            self.update_file_version(connection);
        }
    }

    ///Hash the file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        self.hash = Some(sea_hash(&self.full_path));
    }

    pub fn verify_own_hash(&mut self) -> bool {
        if self.fast_hash.is_some() {
            return self.fast_hash.as_ref().unwrap().as_str() == sea_hash(&self.full_path).as_str();
        } else {
            warn!("This file has nothing to verify it's hash against. Hashing file instead, this will become the hash and will return true.");
            self.hash();
            true
        }
    }

    ///Returns true if hashes match, false if not
    pub fn verify_hash_against(&mut self, path: &Path) -> bool {
        //Note: This could loop infinitely if self.hash isn't set
        if self.hash.is_some() {
            return self.hash.as_ref().unwrap().as_str() == sea_hash(path).as_str();
        } else {
            warn!("Fast hash verification was run on a file without a hash. Hashing file instead, then comparing.");
            self.hash();
            self.verify_hash_against(path)
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
        self.fast_hash = Some(sea_fast_hash(&self.full_path));
    }

    pub fn verify_own_fast_hash(&mut self) -> bool {
        if self.fast_hash.is_some() {
            return self.fast_hash.as_ref().unwrap().as_str()
                == sea_fast_hash(&self.full_path).as_str();
        } else {
            warn!("This file has nothing to verify it's hash against. Hashing file instead, this will become the hash and will return true.");
            self.fast_hash();
            true
        }
    }

    ///Returns true if hashes match, false if not
    pub fn verify_fast_hash_against(&mut self, path: &Path) -> bool {
        if self.fast_hash.is_some() {
            return self.fast_hash.as_ref().unwrap().as_str() == sea_fast_hash(path).as_str();
        } else {
            warn!("Fast hash verification was run on a file without a hash. Hashing file instead, then comparing.");
            self.fast_hash();
            self.verify_fast_hash_against(path)
        }
    }

    pub fn get_file_name(&self) -> String {
        get_file_name(&self.full_path)
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

    pub fn generate_file_version_profiles_if_none(&mut self, connection: &PgConnection) {
        for file_version in self.file_versions.iter_mut() {
            file_version.generate_profile_if_none(connection)
        }
    }

    pub fn insert_file_version(&mut self, file_version: FileVersion) {
        self.file_versions.push(file_version)
    }

    pub fn update_hashes_from_file_versions(&mut self, file_versions: &[FileVersion]) {
        for file_version in self.file_versions.iter_mut() {
            for new_file_version in file_versions {
                if new_file_version.id == file_version.id {
                    file_version.fast_hash = new_file_version.fast_hash.clone();
                    file_version.hash = new_file_version.hash.clone();
                }
            }
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
pub fn sea_hash(full_path: &Path) -> String {
    let mut buffer = Box::new(vec![0; 4096]);
    let mut hasher = seahash::SeaHasher::new();
    let mut file = File::open(pathbuf_to_string(full_path)).unwrap_or_else(|err| {
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
pub fn sea_fast_hash(full_path: &Path) -> String {
    let mut buffer = Box::new(vec![0; 4096]);
    let mut hasher = seahash::SeaHasher::default();
    let mut file = File::open(pathbuf_to_string(full_path)).unwrap_or_else(|err| {
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
