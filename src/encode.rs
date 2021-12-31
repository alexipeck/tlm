use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{Arc, RwLock},
};
use tracing::{error, info, debug};

use crate::{
    generic::FileVersion, pathbuf_file_stem, pathbuf_to_string,
    pathbuf_with_suffix, config::ServerConfig, pathbuf_file_name,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub generic_uid: i32,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub temp_target_path: PathBuf,
    pub encode_string: EncodeString,
}

impl Encode {
    pub fn new(file_version: &FileVersion, encode_profile: &EncodeProfile, server_config: &Arc<RwLock<ServerConfig>>) -> Self {
        let target_path = generate_target_path(&file_version.full_path, encode_profile);
        let temp_target_path = server_config.read().unwrap().tracked_directories.get_temp_directory().join(pathbuf_file_name(&target_path));
        Self {
            generic_uid: file_version.generic_uid,
            source_path: file_version.full_path.clone(),
            target_path,
            temp_target_path,
            encode_string: EncodeString::generate(file_version, encode_profile),
        }
    }

    pub fn get_worker_temp_target_path(&self) -> PathBuf {
        self.encode_string.get_worker_temp_target_path()
    }

    pub fn run(self, handle: Arc<RwLock<Option<Child>>>) {
        info!(
            "Encoding file \'{}\'",
            pathbuf_to_string(&pathbuf_file_name(&self.source_path)),
        );
        debug!("Encode: Source: {}", pathbuf_to_string(&self.source_path));
        debug!("Encode: Destination: {}", self.encode_string.encode_string[&self.encode_string.encode_string.len() - 1]);

        let _ = handle.write().unwrap().insert(
            Command::new("ffmpeg")
                .args(&self.encode_string.get_encode_string())
                .spawn()
                .unwrap(),
        );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodeString {
    //This field is specifically not public, it should never be used without being this EncodeString from being activated
    encode_string: Vec<String>,
    file_name: PathBuf,
    extension: PathBuf,
    worker_temp_full_path: Option<PathBuf>,
}

///Returns a vector of ffmpeg arguments for later execution
///This has no options currently
impl EncodeString {
    pub fn generate(file_version: &FileVersion, encode_profile: &EncodeProfile) -> Self {
        let mut encode_string = vec!["-i".to_string(), pathbuf_to_string(&file_version.full_path)];

        let extension: PathBuf = PathBuf::from(encode_profile.get_extension());

        //TODO: Add settings based on the profile used
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
        //encode_string.push(pathbuf_to_string(&generate_target_path(&file_version.full_path)));
        Self {
            encode_string,
            file_name: pathbuf_file_stem(&file_version.full_path),
            extension,
            worker_temp_full_path: None,
        }
    }

    pub fn get_worker_temp_target_path(&self) -> PathBuf {
        match self.worker_temp_full_path.as_ref() {
            Some(worker_temp_full_path) => {
                worker_temp_full_path.clone()
            },
            None => {
                error!("This should not be none, something has gone very wrong.");
                panic!();
            },
        }
    }

    pub fn get_encode_string(&self) -> Vec<String> {
        if self.is_activated() {
            self.encode_string.clone()
        } else {
            error!("get_encode_string was called before this encode was activated.");
            panic!();
        }
    }

    pub fn is_activated(&self) -> bool {
        self.worker_temp_full_path.is_some()
    }

    pub fn activate(&mut self, temp_path: PathBuf) {
        if self.worker_temp_full_path.is_some() {
            error!("Activate was called on an EncodeString that has already been activated");
            panic!();
        }
        debug!("1: {}", pathbuf_to_string(&temp_path));
        debug!("2: {}", pathbuf_to_string(&self.file_name));
        debug!("3: {}", pathbuf_to_string(&self.extension));
        let mut temp_file_name = self.file_name.clone();
        let _ = temp_file_name.set_extension(&self.extension);
        debug!("4: {}", pathbuf_to_string(&temp_file_name));
        let t = temp_path.join(temp_file_name);
        debug!("5: {}", pathbuf_to_string(&t));
        let worker_temp_full_path = generate_temp_target_path(&t);
        debug!("6: {}", pathbuf_to_string(&worker_temp_full_path));
        self.encode_string.push(pathbuf_to_string(&worker_temp_full_path));
        self.worker_temp_full_path = Some(worker_temp_full_path);
    }
}

pub fn generate_temp_target_path(full_path: &Path) -> PathBuf {
    //every temp file that is added should be tracked and if it receives no action,
    //it should be deleted to ensure the program doesn't use more storage than needed
    pathbuf_with_suffix(full_path, "_temp".to_string())
}

#[derive(Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum EncodeProfile {
    H264_TV_1080p,
    H265_TV_1080p,
}

impl EncodeProfile {
    //TODO: Make realistic association between profile and container
    pub fn get_extension(&self) -> String {
        match self {
            EncodeProfile::H264_TV_1080p => "mp4".to_string(),
            EncodeProfile::H265_TV_1080p => "mp4".to_string(),
        }
    }
}

impl fmt::Display for EncodeProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncodeProfile::H264_TV_1080p => write!(f, "H264_TV_1080p"),
            EncodeProfile::H265_TV_1080p => write!(f, "H265_TV_1080p"),
        }
    }
}

//total_versions_inclusive refers to the total number of versions there will be for a file after an encode
//TODO: it needs to take into account if multiple encodes have been started for one particular file
pub fn generate_target_path(full_path: &Path, encode_profile: &EncodeProfile) -> PathBuf {
    //TODO: This function should create the actual target path for a new FileVersion
    //TODO: Check that there isn't already a file with that file name
    pathbuf_with_suffix(full_path, format!("_{}", encode_profile.to_string()))
}
