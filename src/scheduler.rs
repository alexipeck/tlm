use crate::{
    config::{Config, Preferences},
    database::establish_connection,
    diesel::SaveChangesDsl,
    generic::Generic,
    manager::FileManager,
    model::GenericModel,
    profile::Profile,
};
use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use std::{
    collections::VecDeque,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    time,
};

static TASK_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)] //, Serialize, Deserialize
pub struct Worker {
    encode_queue: Arc<Mutex<VecDeque<Task>>>,
    pub ws_tx: UnboundedSender<tokio_tungstenite::tungstenite::Message>,
    //TODO: Store the HashMap/tx
    //TODO: Worker UID, should be based on some hardware identifier, so it can be regenerated
    //NOTE: If this is running under a Docker container, it may have a random MAC address, so on reboot,
    //    : becoming a new worker will probably mean the old one should be dropped after x amount of time.

    //TODO: websocket connection handle
}

impl Worker {
    pub fn new(
        ws_tx: UnboundedSender<tokio_tungstenite::tungstenite::Message>,
        encode_tasks: Arc<Mutex<VecDeque<Task>>>,
        queue_capacity: u32,
    ) -> Self {
        let temp = Self {
            encode_queue: Arc::new(Mutex::new(VecDeque::new())),
            ws_tx,
        };
        for _ in 0..queue_capacity {
            match encode_tasks.lock().unwrap().pop_front() {
                Some(encode_task) => {
                    temp.encode_queue
                        .lock()
                        .unwrap()
                        .push_back(encode_task);
                },
                None => {info!("No encode tasks to send to the worker")},
            }
        }
        temp
    }

    pub fn check_if_active(&mut self) {
        //If the connection is active, do nothing.
        //TODO: If something is wrong with the connection, close the connection server-side, making this worker unavailable, start timeout for removing the work assigned to the worker.
        //    : If there's not enough work for other workers, immediately shift the encodes to other workers (put it back in the encode queue)
    }
}

///Struct to represent a file encode task. This is needed so we can have an enum
///that contains all types of task
///This should probably handle it's current variables without having them passed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub source_path: PathBuf,
    pub future_filename: String,
    pub encode_options: Vec<String>,
    pub profile: Profile,
}

impl Encode {
    pub fn new(
        source_path: PathBuf,
        future_filename: String,
        encode_options: Vec<String>,
        profile: Profile,
    ) -> Self {
        Encode {
            source_path,
            future_filename,
            encode_options,
            profile,
        }
    }

    ///Run the encode TODO: Make this task asynchronous, allow offloading to worker machine
    //TODO: This will need to use the link to the worker
    pub fn send_to_worker(&mut self) {
        //share credentials will have to be handled on the worker side
        if !self.source_path.exists() {
            //TODO: mark this Encode Task as failed because "file not found", change it to a
            //      state where it can be stored, then manually fixed before being restarted,
            return;
        }
        //TODO: Have the worker send a message to the server if it can't access the file
    }

    pub fn run(&mut self) {
        info!(
            "Encoding file \'{}\'",
            Generic::get_filename_from_pathbuf(self.source_path.clone())
        );

        let _buffer = Command::new("ffmpeg")
            .args(&self.encode_options.clone())
            .output()
            .unwrap_or_else(|err| {
                error!("Failed to execute ffmpeg process. Err: {}", err);
                panic!();
            });

        //only uncomment if you want disgusting output
        //should be error, but from ffmpeg, stderr mostly consists of stdout information
        //print(Verbosity::DEBUG, "generic", "encode", format!("{}", String::from_utf8_lossy(&buffer.stderr).to_string()));
    }
}

///Struct to represent a file import task. This is needed so we can have an enum
///that contains all types of task
#[derive(Clone, Debug)]
pub struct ImportFiles {}

impl ImportFiles {
    pub fn run(&mut self, file_manager: &mut FileManager) {
        info!("Started importing new files");
        file_manager.import_files();
        info!("Finished importing new files");
    }
}

impl Default for ImportFiles {
    fn default() -> Self {
        ImportFiles {}
    }
}

///Struct to represent a file processing task. This is needed so we can have an enum
///that contains all types of task
#[derive(Clone, Debug)]
pub struct ProcessNewFiles {}

impl ProcessNewFiles {
    pub fn run(&mut self, file_manager: &mut FileManager, preferences: &Preferences) {
        info!("Started processing new files");
        file_manager.process_new_files(preferences);
        info!("Finished processing new files you can now stop the program with Ctrl-c");
    }
}

impl Default for ProcessNewFiles {
    fn default() -> Self {
        ProcessNewFiles {}
    }
}

///Struct to represent a hashing task. This is needed so we can have an enum
///that contains all types of task.
#[derive(Clone, Debug)]
pub struct Hash {}

impl Hash {
    pub fn run(&self, current_content: Vec<Generic>) -> TaskReturnAsync {
        let is_finished = Arc::new(AtomicBool::new(false));

        info!("Started hashing in the background");
        let is_finished_inner = is_finished.clone();
        //Hash files until all other functions are complete
        let handle = Some(thread::spawn(move || {
            let mut current_content = current_content;
            current_content.retain(|elem| elem.hash.is_none() || elem.fast_hash.is_none());
            let length = current_content.len();
            let connection = establish_connection();
            let mut did_finish = true;
            for (i, content) in current_content.iter_mut().enumerate() {
                if content.hash.is_none() {
                    content.hash();
                    debug!(
                        "Hashed[{} of {}]: {}",
                        i + 1,
                        length,
                        content.full_path.to_str().unwrap()
                    );
                    content.fast_hash();
                    if GenericModel::from_generic(content.clone())
                        .save_changes::<GenericModel>(&connection)
                        .is_err()
                    {
                        error!("Failed to update hash in database");
                    }
                } else if content.fast_hash.is_none() {
                    content.fast_hash();
                    if GenericModel::from_generic(content.clone())
                        .save_changes::<GenericModel>(&connection)
                        .is_err()
                    {
                        error!("Failed to update hash in database");
                    }
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

impl Default for Hash {
    fn default() -> Self {
        Hash {}
    }
}

///This enum is required to create a queue of tasks independent of task type
#[derive(Clone, Debug)]
pub enum TaskType {
    Encode(Encode),
    ImportFiles(ImportFiles),
    ProcessNewFiles(ProcessNewFiles),
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

    ///execute the tasks run function
    pub fn handle_task(
        &mut self,
        file_manager: &mut FileManager,
        preferences: &Preferences,
    ) -> Option<TaskReturnAsync> {
        match &mut self.task_type {
            TaskType::Encode(encode) => {
                //TODO: Start tracking a the worker that will be assigned to an encode (includes next in queue), if it isn't already
                //TODO: If the Task is added as the next in line encode for the worker, change it's status to "waiting for encode"
                //TODO:
                encode.send_to_worker();
            }
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager);
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager, preferences);
            }
            TaskType::Hash(hash) => {
                let mut current_content = file_manager.generic_files.clone();
                for show in &file_manager.shows {
                    for season in &show.seasons {
                        for episode in &season.episodes {
                            current_content.push(episode.generic.clone());
                        }
                    }
                }
                return Some(hash.run(current_content));
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
    pub file_manager: FileManager,
    pub tasks: Arc<Mutex<VecDeque<Task>>>,
    pub encode_tasks: Arc<Mutex<VecDeque<Task>>>,
    pub active_workers: Arc<Mutex<Vec<Worker>>>,
    pub config: Config,
    pub input_completed: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new(
        config: Config,
        tasks: Arc<Mutex<VecDeque<Task>>>,
        encode_tasks: Arc<Mutex<VecDeque<Task>>>,
        active_workers: Arc<Mutex<Vec<Worker>>>,
        input_completed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            tasks,
            encode_tasks,
            file_manager: FileManager::new(&config),
            config,
            active_workers,
            input_completed,
        }
    }

    pub fn start_scheduler(&mut self, preferences: &Preferences) {
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
                let _handle = handles[i].handle.take();
                _handle.unwrap().join();
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

            let result = task.handle_task(&mut self.file_manager, preferences);
            if let Some(handle) = result {
                handles.push(handle);
            }
        }
    }
}
