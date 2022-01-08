use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};

use tracing::{debug, error, info};

use crate::{
    config::ServerConfig,
    copy,
    database::create_file_version,
    encode::Encode,
    file_manager::FileManager,
    generic::FileVersion,
    model::NewFileVersion,
    pathbuf_to_string, remove_file,
    scheduler::{GenerateProfiles, Hash, ImportFiles, ProcessNewFiles, Task, TaskType},
    worker::WorkerMessage,
    worker_manager::{AddEncodeMode, WorkerManager},
    PeerMap, Tx,
};

pub fn import_files(tasks: Arc<Mutex<VecDeque<Task>>>) {
    tasks
        .lock()
        .unwrap()
        .push_back(Task::new(TaskType::ImportFiles(ImportFiles::default())))
}

pub fn process_files(tasks: Arc<Mutex<VecDeque<Task>>>) {
    tasks
        .lock()
        .unwrap()
        .push_back(Task::new(TaskType::ProcessNewFiles(
            ProcessNewFiles::default(),
        )))
}

pub fn hash_files(tasks: Arc<Mutex<VecDeque<Task>>>) {
    tasks
        .lock()
        .unwrap()
        .push_back(Task::new(TaskType::Hash(Hash::default())))
}

pub fn generate_profiles(tasks: Arc<Mutex<VecDeque<Task>>>) {
    tasks
        .lock()
        .unwrap()
        .push_back(Task::new(TaskType::GenerateProfiles(
            GenerateProfiles::default(),
        )));
}

//WorkerMessage functions
pub fn initialise(
    initialise_message: WorkerMessage,
    worker_manager: Arc<Mutex<WorkerManager>>,
    addr: SocketAddr,
    tx: Tx,
    peer_map: Arc<Mutex<PeerMap>>,
) {
    if let WorkerMessage::Initialise(mut worker_uid, worker_temp_directory) = initialise_message {
        debug!("Init worker: {}", pathbuf_to_string(&worker_temp_directory));
        //if true {//TODO: authenticate/validate
        if !worker_manager.lock().unwrap().reestablish_worker(
            worker_uid,
            addr,
            tx.clone(),
        ) {
            //We need the new uid so we can set it correctly in the peer map
            worker_uid = Some(worker_manager.lock().unwrap().add_worker(
                addr,
                tx,
            ));
        }
        peer_map.lock().unwrap().get_mut(&addr).unwrap().0 = worker_uid;
        //}
    } else {
        panic!();
    }
}

pub fn encode_generic(
    encode_generic_message: WorkerMessage,
    file_manager: Arc<Mutex<FileManager>>,
    worker_manager_transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    server_config: Arc<RwLock<ServerConfig>>,
) {
    if let WorkerMessage::EncodeGeneric(
        generic_uid,
        file_version_id,
        add_encode_mode,
        encode_profile,
    ) = encode_generic_message
    {
        match file_manager.lock().unwrap().get_encode_from_generic_uid(
            generic_uid,
            file_version_id,
            &encode_profile,
            server_config,
        ) {
            Some(encode) => {
                match add_encode_mode {
                    AddEncodeMode::Back => {
                        worker_manager_transcode_queue
                            .lock()
                            .unwrap()
                            .push_back(encode);
                    }
                    AddEncodeMode::Next => {
                        worker_manager_transcode_queue
                            .lock()
                            .unwrap()
                            .push_front(encode);
                    }
                    AddEncodeMode::Now => {
                        //TODO: Implement immediate encode
                    }
                }
                info!("Setting up generic for transcode");
            }
            None => {
                info!("No generics available to transcode");
            }
        }
    } else {
        panic!();
    }
}

pub fn encode_started(encode_started_message: WorkerMessage) {
    if let WorkerMessage::EncodeStarted(worker_uid, generic_uid) = encode_started_message {
        info!(
            "Worker with UID: {} has started transcoding generic with UID: {}",
            worker_uid, generic_uid,
        );
    } else {
        panic!();
    }
}

pub fn encode_finished(encode_finished_message: WorkerMessage) {
    if let WorkerMessage::EncodeFinished(worker_uid, generic_uid, full_path) =
        encode_finished_message
    {
        info!(
            "Worker with UID: {} has finished transcoding file with generic_uid: {}, worker file system location: {}",
            worker_uid,
            generic_uid,
            pathbuf_to_string(&full_path),
        );
    } else {
        panic!();
    }
}

pub fn move_started(move_started_message: WorkerMessage) {
    if let WorkerMessage::MoveStarted(
        worker_uid,
        generic_uid,
        remote_source_path,
        destination_path,
    ) = move_started_message
    {
        info!(
            "Worker with UID: {} has started moving file with generic_uid: {}, from: \"{}\" to \"{}\"",
            worker_uid,
            generic_uid,
            pathbuf_to_string(&remote_source_path),
            pathbuf_to_string(&destination_path),
        );
    } else {
        panic!();
    }
}

pub fn move_finished(
    move_finished_message: WorkerMessage,
    worker_manager: Arc<Mutex<WorkerManager>>,
    file_manager: Arc<Mutex<FileManager>>,
) {
    if let WorkerMessage::MoveFinished(worker_uid, generic_uid, encode) = move_finished_message {
        if let Err(err) = copy(&encode.temp_target_path, &encode.target_path) {
            error!(
                "Failed to copy file from server temp to media library. IO output: {}",
                err
            );
            panic!();
        }
        if let Err(err) = remove_file(&encode.temp_target_path) {
            error!("Failed to remove file from server temp. IO output: {}", err);
            panic!();
        }
        //TODO: Make this whole process persistent
        worker_manager
            .lock()
            .unwrap()
            .clear_current_transcode_from_worker(worker_uid, generic_uid);
        if !file_manager
            .lock()
            .unwrap()
            .insert_file_version(&FileVersion::from_file_version_model(create_file_version(
                NewFileVersion::new(generic_uid, pathbuf_to_string(&encode.target_path), false),
            )))
        {
            error!(
                "This should've found a generic to insert it into, this shouldn't have happened."
            );
            panic!();
        }
        //TODO: Make an enum of actions that could be performed on a Worker, like clear_current_transcode
    } else {
        panic!();
    }
}
