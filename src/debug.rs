use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tracing::debug;

use crate::{
    encode::{Encode, EncodeProfile},
    file_manager::FileManager,
    generic::FileVersion,
    to_string,
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
    fn line_output(file_version: &FileVersion) {
        let mut missing_fields: String = String::new();
        if file_version.hash.is_none() {
            missing_fields.push_str(", hash");
        }
        if file_version.fast_hash.is_none() {
            missing_fields.push_str(", fast_hash");
        }
        if file_version.width.is_none() {
            missing_fields.push_str(", width");
        }
        if file_version.height.is_none() {
            missing_fields.push_str(", height");
        }
        if file_version.framerate.is_none() {
            missing_fields.push_str(", framerate");
        }
        if file_version.length_time.is_none() {
            missing_fields.push_str(", length_time");
        }
        if file_version.resolution_standard.is_none() {
            missing_fields.push_str(", resolution_standard");
        }
        if file_version.container.is_none() {
            missing_fields.push_str(", container");
        }
        if !missing_fields.is_empty() {
            debug!(
                "File Version: generic_uid: {}, id: {} is missing: {}",
                file_version.generic_uid,
                file_version.id,
                missing_fields.replacen(", ", "", 1)
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
            to_string(tracked_directory)
        );
    }
    debug!(
        "Cache directory: {}",
        to_string(
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
