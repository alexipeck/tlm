use crate::{
    config::ServerConfig,
    database::{establish_connection, update_file_version},
    file_manager::FileManager,
    generic::FileVersion,
    pathbuf_to_string,
};
use tracing::{debug, error, info};

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        RwLock,
    },
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    time,
};

static TASK_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

///Struct to represent a file import task. This is needed so we can have an enum
///that contains all types of task
#[derive(Clone, Debug, Default)]
pub struct ImportFiles {}

impl ImportFiles {
    pub fn run(&mut self, file_manager: Arc<Mutex<FileManager>>) {
        info!("Started importing new files");
        file_manager.lock().unwrap().import_files();
        info!("Finished importing new files");
    }
}

///Struct to represent a file processing task. This is needed so we can have an enum
///that contains all types of task
#[derive(Clone, Debug, Default)]
pub struct ProcessNewFiles {}

impl ProcessNewFiles {
    pub fn run(&mut self, file_manager: Arc<Mutex<FileManager>>) {
        info!("Started processing new files");
        file_manager.lock().unwrap().process_new_files();
        info!("Finished processing new files");
    }
}

///Struct to represent a file processing task. This is needed so we can have an enum
///that contains all types of task
#[derive(Clone, Debug, Default)]
pub struct GenerateProfiles {}

impl GenerateProfiles {
    pub fn run(&mut self, file_manager: Arc<Mutex<FileManager>>) {
        info!("Started generating profiles");
        file_manager.lock().unwrap().generate_profiles();
        info!("Finished generating profiles");
    }
}

///Struct to represent a hashing task. This is needed so we can have an enum
///that contains all types of task.
#[derive(Clone, Debug, Default)]
pub struct Hash {}

impl Hash {
    pub fn run(&self, file_manager: Arc<Mutex<FileManager>>) -> TaskReturnAsync {
        let is_finished = Arc::new(AtomicBool::new(false));
        let mut generic_file_versions: Vec<(i32, Vec<FileVersion>)> = Vec::new();
        let mut file_version_count = 0;
        //Collect FileVersions for hashing from generic_files
        for generic in file_manager.lock().unwrap().generic_files.iter() {
            if generic.has_hashing_work() {
                generic_file_versions
                    .push((generic.get_generic_uid(), generic.file_versions.clone()));
                file_version_count += generic.file_versions.len();
            }
        }

        //Collect FileVersions for hashing from episodes
        let mut episode_file_versions: Vec<(i32, Vec<FileVersion>)> = Vec::new();
        for show in &file_manager.lock().unwrap().shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    if episode.generic.has_hashing_work() {
                        episode_file_versions.push((
                            episode.generic.get_generic_uid(),
                            episode.generic.file_versions.clone(),
                        ));
                        file_version_count += episode.generic.file_versions.len();
                    }
                }
            }
        }

        info!("Started hashing in the background");
        let is_finished_inner = is_finished.clone();
        //Hash files until all other functions are complete
        let handle = Some(thread::spawn(move || {
            let mut did_finish = true;
            let connection = establish_connection();
            let mut current_file_version_count = 0;
            //TODO: Switch the output below to use only one function for output
            //Generics
            for (generic_uid, file_versions) in generic_file_versions.iter_mut() {
                let length: usize = file_versions.len();
                for (i, file_version) in file_versions.iter_mut().enumerate() {
                    if file_version.hash.is_none() {
                        file_version.hash();
                    }
                    if file_version.fast_hash.is_none() {
                        file_version.fast_hash();
                    }
                    debug!(
                        "Hashed[{:2} of {:2}][{} of {}]: {}",
                        current_file_version_count + 1,
                        file_version_count,
                        i + 1,
                        length,
                        pathbuf_to_string(&file_version.full_path)
                    );
                    update_file_version(file_version, &connection);
                    current_file_version_count += 1;
                }
                //Update the generic in ram, if it has been deleted then don't worry about it
                for live_generic in file_manager.lock().unwrap().generic_files.iter_mut() {
                    if live_generic.generic_uid == Some(*generic_uid) {
                        live_generic.update_hashes_from_file_versions(file_versions);
                    }
                }
                if is_finished_inner.load(Ordering::Relaxed) {
                    did_finish = false;
                    break;
                }
            }
            //Episodes
            for (generic_uid, file_versions) in episode_file_versions.iter_mut() {
                let length: usize = file_versions.len();
                for (i, file_version) in file_versions.iter_mut().enumerate() {
                    if file_version.hash.is_none() {
                        file_version.hash();
                    }
                    if file_version.fast_hash.is_none() {
                        file_version.fast_hash();
                    }
                    debug!(
                        "Hashed[{:2} of {:2}][{} of {}]: {}",
                        current_file_version_count + 1,
                        file_version_count,
                        i + 1,
                        length,
                        pathbuf_to_string(&file_version.full_path)
                    );
                    update_file_version(file_version, &connection);
                    current_file_version_count += 1;
                }
                let mut found_generic = false;
                for show in file_manager.lock().unwrap().shows.iter_mut() {
                    for season in show.seasons.iter_mut() {
                        for episode in season.episodes.iter_mut() {
                            if episode.generic.generic_uid == Some(*generic_uid) {
                                episode
                                    .generic
                                    .update_hashes_from_file_versions(file_versions);
                                found_generic = true;
                                break;
                            }
                        }
                    }
                }
                if !found_generic {
                    error!("Couldn't find Episode->Generic to update the hash of in memory");
                    panic!();
                }
                if is_finished_inner.load(Ordering::Relaxed) {
                    did_finish = false;
                    break;
                }
            }

            is_finished_inner.store(true, Ordering::Relaxed);
            if did_finish {
                info!("Finished hashing");
            } else {
                info!("Stopped hashing (incomplete)");
            }
        }))
        .unwrap();

        TaskReturnAsync::new(Some(handle), is_finished)
    }
}

///This enum is required to create a queue of tasks independent of task type
#[derive(Clone, Debug)]
pub enum TaskType {
    ImportFiles(ImportFiles),
    ProcessNewFiles(ProcessNewFiles),
    GenerateProfiles(GenerateProfiles),
    Hash(Hash),
}

///Task struct that will later be in the database with a real id so that the queue
///persists between runs
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Task {
    task_uid: usize,
    task_type: TaskType,
}

impl Task {
    pub fn new(task_type: TaskType) -> Self {
        Task {
            task_uid: TASK_UID_COUNTER.fetch_add(1, Ordering::SeqCst), //TODO: Store in the database
            task_type,
        }
    }

    //TODO: When the server stops or wants to check whether a long time running thread is actually still doing work, it should be able to access that handle, and perform some form of check that determines whether it expects it to still be running, like .is_running(), is_ok(), etc on the handle with a level of severity, determines what it is and whether it is actually working, this will somehow involve it communicating with it.

    ///execute the tasks run function
    pub fn handle_task(
        &mut self,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Option<TaskReturnAsync> {
        match &mut self.task_type {
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager);
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager);
            }
            TaskType::Hash(hash) => {
                return Some(hash.run(file_manager));
            }
            TaskType::GenerateProfiles(generate_profiles) => {
                generate_profiles.run(file_manager);
            }
        }
        None
    }
}

///Struct to store all data required by an asynchronous task in order to join it and know when it has
///finished it's task or tell it to stop early
pub struct TaskReturnAsync {
    handle: Option<JoinHandle<()>>,
    is_done: Arc<AtomicBool>,
}

impl TaskReturnAsync {
    pub fn new(handle: Option<JoinHandle<()>>, is_done: Arc<AtomicBool>) -> Self {
        Self { handle, is_done }
    }
}

///Schedules all tasks and contains a queue of tasks that can be modified by other threads
pub struct Scheduler {
    pub file_manager: Arc<Mutex<FileManager>>,
    pub tasks: Arc<Mutex<VecDeque<Task>>>,
    pub encode_tasks: Arc<Mutex<VecDeque<Task>>>,
    pub config: Arc<RwLock<ServerConfig>>,
    pub input_completed: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new(
        config: Arc<RwLock<ServerConfig>>,
        tasks: Arc<Mutex<VecDeque<Task>>>,
        encode_tasks: Arc<Mutex<VecDeque<Task>>>,
        file_manager: Arc<Mutex<FileManager>>,
        input_completed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            tasks,
            encode_tasks,
            file_manager,
            config,
            input_completed,
        }
    }

    pub fn start_scheduler(&mut self) {
        let wait_time = time::Duration::from_secs(1);

        //Take a handle from any async function and a booleans
        //The Handle is in an option so we can take the Handle in order to join it,
        //that is necessary because otherwise it is owned by the vector and joining would destroy it
        //The boolean tells the thread to stop and tells the scheduler thread that it has stopped
        //Essentially it just makes it safe to join
        let mut handles: Vec<TaskReturnAsync> = Vec::new();
        loop {
            let mut task: Task;

            //Mark the completed threads
            let mut completed_threads: Vec<usize> = Vec::new();
            for (i, handle) in handles.iter().enumerate() {
                if handle.is_done.load(Ordering::Relaxed) {
                    //opts.0.join();
                    completed_threads.push(i);
                }
            }

            //Reverse so we can remove items right to left avoiding the index shifting
            completed_threads.reverse();

            //Take each completed thread and join it
            for i in completed_threads {
                let handle = handles[i].handle.take();
                if let Err(err) = handle.unwrap().join() {
                    error!("{:?}", err);
                }
                handles.remove(i);
            }
            {
                let mut tasks = self.tasks.lock().unwrap();

                //When the queue is empty we wait until another item is added or user input is marked as completed
                if tasks.len() == 0 {
                    if self.input_completed.load(Ordering::Relaxed) {
                        //Tell all async tasks to stop early if they can
                        for handle in handles {
                            handle.is_done.store(true, Ordering::Relaxed);
                            let _res = handle.handle.unwrap().join();
                        }
                        break;
                    }
                    std::mem::drop(tasks); //Unlock the Mutex so we don't block while sleeping
                    thread::sleep(wait_time);
                    continue;
                }
                task = tasks.pop_front().unwrap();
            }

            let result = task.handle_task(self.file_manager.clone());
            if let Some(handle) = result {
                handles.push(handle);
            }
        }
    }
}
