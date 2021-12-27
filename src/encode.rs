use std::{path::{PathBuf, Path}, sync::{Arc, RwLock}, process::{Child, Command}};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{pathbuf_file_name_to_string, generic::FileVersion, pathbuf_to_string, pathbuf_with_suffix};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub generic_uid: i32,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub encode_options: Vec<String>,
}

impl Encode {
    pub fn new(
        file_version: &FileVersion,
    ) -> Self {
        Self {
            generic_uid: file_version.generic_uid,
            source_path: file_version.full_path.clone(),
            target_path: generate_target_path(&file_version.full_path),
            encode_options: generate_encode_string(file_version),
        }
        //asdf;
    }

    pub fn run(self, handle: Arc<RwLock<Option<Child>>>) {
        info!(
            "Encoding file \'{}\'",
            pathbuf_file_name_to_string(&self.source_path)
        );

        let _ = handle.write().unwrap().insert(
            Command::new("ffmpeg")
                .args(&self.encode_options)
                .spawn()
                .unwrap(),
        );
    }
}

///Returns a vector of ffmpeg arguments for later execution
///This has no options currently
pub fn generate_encode_string(file_version: &FileVersion) -> Vec<String> {
    let mut encode_string = vec!["-i".to_string(), pathbuf_to_string(&file_version.full_path)];

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
    encode_string.push(pathbuf_to_string(&generate_target_path(&file_version.full_path)));
    encode_string
}

///Appends a fixed string to differentiate rendered files from original before overwrite
/// I doubt this will stay as I think a temp directory would be more appropriate.
/// This function returns that as a string for the ffmpeg arguments
pub fn generate_target_path(full_path: &Path) -> PathBuf {
    pathbuf_with_suffix(full_path, format!(
        "_temp_test_encode{}",
        rand::thread_rng().gen::<i32>()
    ))
}