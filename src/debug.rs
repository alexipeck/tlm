use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tracing::debug;

use crate::{
    encode::{Encode, EncodeProfile},
    file_manager::FileManager,
    generic::FileVersion,
    pathbuf_to_string,
};

pub fn output_all_file_versions(file_manager: Arc<Mutex<FileManager>>) {
    debug!("Start outputting all file versions:");
    let file_manager_lock = file_manager.lock().unwrap();
    for generic in file_manager_lock.generic_files.iter() {
        for file_version in generic.file_versions.iter() {
            debug!(
                "generic_uid: {:3}, file_version_id: {:3}, filename: {}",
                file_version.generic_uid,
                file_version.id,
                file_version.get_filename()
            );
        }
    }
}

pub fn run_completeness_check(file_manager: Arc<Mutex<FileManager>>) {
    fn bool_to_char(bool: bool) -> char {
        if bool {
            'Y'
        } else {
            'N'
        }
    }
    fn line_output(file_version: &FileVersion) {
        let hash = file_version.hash.is_some();
        let fast_hash = file_version.fast_hash.is_some();
        let width = file_version.width.is_some();
        let height = file_version.height.is_some();
        let framerate = file_version.framerate.is_some();
        let length_time = file_version.length_time.is_some();
        let resolution_standard = file_version.resolution_standard.is_some();
        let container = file_version.container.is_some();
        if !hash
            || !fast_hash
            || !width
            || !height
            || !framerate
            || !length_time
            || !resolution_standard
            || !container
        {
            debug!(
                "hash: {}, fast_hash: {}, width: {}, height: {}, framerate: {}, length_time: {}, resolution_standard: {}, container: {}",
                bool_to_char(hash),
                bool_to_char(fast_hash),
                bool_to_char(width),
                bool_to_char(height),
                bool_to_char(framerate),
                bool_to_char(length_time),
                bool_to_char(resolution_standard),
                bool_to_char(container),
            );
        }
    }
    debug!("Starting completeness check");
    let file_manager_lock = file_manager.lock().unwrap();

    debug!("Generics: {}", file_manager_lock.generic_files.len());
    let mut episodes_count = 0;
    for show in file_manager_lock.shows.iter() {
        for season in show.seasons.iter() {
            for episode in season.episodes.iter() {
                episodes_count += episode.generic.file_versions.len();
            }
        }
    }
    debug!("Episodes: {}", episodes_count);

    for generic in file_manager_lock.generic_files.iter() {
        for file_version in generic.file_versions.iter() {
            line_output(file_version);
        }
    }
    for show in file_manager_lock.shows.iter() {
        for season in show.seasons.iter() {
            for episode in season.episodes.iter() {
                for file_version in episode.generic.file_versions.iter() {
                    line_output(file_version);
                }
            }
        }
    }
    debug!("Finishing completeness check");
}

pub fn output_tracked_paths(file_manager: Arc<Mutex<FileManager>>) {
    let file_manager_lock = file_manager.lock().unwrap();
    for tracked_directory in file_manager_lock
        .config
        .read()
        .unwrap()
        .tracked_directories
        .get_root_directories()
    {
        debug!(
            "Tracked directory: {}",
            pathbuf_to_string(tracked_directory)
        );
    }
    debug!(
        "Cache directory: {}",
        pathbuf_to_string(
            file_manager_lock
                .config
                .read()
                .unwrap()
                .tracked_directories
                .get_cache_directory()
        )
    );
    //TODO: Add more things to the output, anything that might be pulled from the config file, default generated or manually added.
}

pub fn encode_all_files(
    file_manager: Arc<Mutex<FileManager>>,
    worker_mananger_transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
) {
    for encode in file_manager
        .lock()
        .unwrap()
        .generate_encodes_for_all(&EncodeProfile::H265_TV_1080p)
    {
        worker_mananger_transcode_queue
            .lock()
            .unwrap()
            .push_back(encode);
    }
}
