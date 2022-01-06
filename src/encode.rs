use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    fs::remove_file,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, RwLock},
};
use tracing::{debug, error, info};

use crate::{
    config::ServerConfig, copy, generic::FileVersion, get_file_name, get_file_stem,
    pathbuf_to_string, pathbuf_with_suffix,
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
    pub fn new(
        file_version: &FileVersion,
        encode_profile: &EncodeProfile,
        server_config: &Arc<RwLock<ServerConfig>>,
    ) -> Self {
        let target_path = pathbuf_with_suffix(
            &file_version.full_path,
            format!("_{}", encode_profile.to_string()),
        );
        let temp_target_path = server_config
            .read()
            .unwrap()
            .tracked_directories
            .get_global_temp_directory()
            .join(get_file_name(&pathbuf_with_suffix(
                &target_path,
                "_temp".to_string(),
            )));
        Self {
            generic_uid: file_version.generic_uid,
            source_path: file_version.full_path.clone(),
            target_path,
            temp_target_path,
            encode_string: EncodeString::generate_deactivated(file_version, encode_profile),
        }
    }

    pub fn cache_file(&self) {
        if let Err(err) = copy(
            &self.source_path,
            &PathBuf::from(self.encode_string.get_source_path()),
        ) {
            error!("Failed to copy file to temp. IO output: {}", err);
            panic!();
        }
    }

    pub fn delete_file_cache(&self) {
        if let Err(err) = remove_file(&self.encode_string.get_source_path()) {
            error!("Failed to remove file from temp. IO output: {}", err);
            panic!();
        }

        if let Err(err) = remove_file(&self.encode_string.get_target_path()) {
            error!("Failed to remove file from temp. IO output: {}", err);
            panic!();
        }
    }

    pub fn transfer_encode_to_server_temp(&self) {
        if let Err(err) = copy(
            &PathBuf::from(self.encode_string.get_target_path()),
            &self.temp_target_path,
        ) {
            error!(
                "Failed to copy file from temp to global temp. IO output: {}",
                err
            );
            panic!();
        }
    }

    pub fn run(&self, handle: Arc<RwLock<Option<Child>>>, silent: bool) {
        info!("Encoding file \"{}\"", get_file_name(&self.source_path));
        debug!("Encode: Source: {}", pathbuf_to_string(&self.source_path));
        debug!(
            "Encode: Destination: {}",
            self.encode_string.encode_string[&self.encode_string.encode_string.len() - 1]
        );
        if silent {
            //The two elements include null redirect all output to /dev/null or equivalent
            let _ = handle.write().unwrap().insert(
                Command::new("ffmpeg")
                    .args(&self.encode_string.get_encode_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        } else {
            let _ = handle.write().unwrap().insert(
                Command::new("ffmpeg")
                    .args(&self.encode_string.get_encode_string())
                    .spawn()
                    .unwrap(),
            );
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodeString {
    activated: bool,
    //This field is specifically not public, it should never be used without being this EncodeString from being activated
    encode_string: Vec<String>,
    file_name: String,

    //items needed for activation status
    worker_source_index: usize,
    worker_target_index: usize,
}

//Returns a vector of ffmpeg arguments for later execution
//This has no options currently
impl EncodeString {
    pub fn generate_deactivated(
        file_version: &FileVersion,
        encode_profile: &EncodeProfile,
    ) -> Self {
        let mut encode_string: Vec<String> = vec!["-i".to_string()]; //0

        //Get the index of the source path
        let source_index = encode_string.len();
        encode_string.push(String::new()); //1: to_string(&file_version.full_path)

        //TODO: Add settings based on the profile used
        //Video
        encode_string.push("-c:v".to_string()); //2
        encode_string.push("libx265".to_string()); //3
        encode_string.push("-crf".to_string()); //4
        encode_string.push("25".to_string()); //5
        encode_string.push("-preset".to_string()); //6
        encode_string.push("slower".to_string()); //7
        encode_string.push("-profile:v".to_string()); //8
        encode_string.push("main".to_string()); //9

        //Audio
        encode_string.push("-c:a".to_string()); //10
        encode_string.push("aac".to_string()); //11
        encode_string.push("-q:a".to_string()); //12
        encode_string.push("224k".to_string()); //13

        encode_string.push("-y".to_string()); //14

        //Get the index of the destination path
        let destination_index = encode_string.len();
        encode_string.push(String::new()); //15

        Self {
            activated: false,
            encode_string,
            file_name: format!(
                "{}.{}",
                get_file_stem(&file_version.full_path),
                encode_profile.get_extension()
            ),
            worker_source_index: source_index,
            worker_target_index: destination_index,
        }
    }

    pub fn assign_source_path(&mut self, path: &Path) {
        self.encode_string[self.worker_source_index] = pathbuf_to_string(path);
    }

    pub fn assign_target_path(&mut self, path: &Path) {
        self.encode_string[self.worker_target_index] = pathbuf_to_string(path);
    }

    pub fn get_source_path(&self) -> String {
        self.encode_string[self.worker_source_index].clone()
    }

    pub fn get_target_path(&self) -> String {
        self.encode_string[self.worker_target_index].clone()
    }

    pub fn get_encode_string(&self) -> Vec<String> {
        if self.activated {
            self.encode_string.clone()
        } else {
            error!("get_encode_string was called before this encode was activated.");
            panic!();
        }
    }

    pub fn activate(&mut self, temp_path: PathBuf) {
        if self.activated {
            error!("Activate was called on an EncodeString that has already been activated");
            panic!();
        }
        let temp_path = temp_path.join(&self.file_name);
        self.assign_source_path(&temp_path);
        //every temp file that is added should be tracked and if it receives no action,
        //it should be deleted to ensure the program doesn't use more storage than needed
        self.assign_target_path(&pathbuf_with_suffix(&temp_path, "_temp".to_string()));
        self.activated = true;
    }
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
            Self::H264_TV_1080p => write!(f, "H264_TV_1080p"),
            Self::H265_TV_1080p => write!(f, "H265_TV_1080p"),
        }
    }
}

//total_versions_inclusive refers to the total number of versions there will be for a file after an encode
//TODO: it needs to take into account if multiple encodes have been started for one particular file
pub fn generate_target_path(full_path: &Path, encode_profile: &EncodeProfile) -> PathBuf {
    //TODO: This function should create the actual target path for a new FileVersion

    pathbuf_with_suffix(full_path, format!("_{}", encode_profile.to_string()))
}
