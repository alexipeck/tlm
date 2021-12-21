use crate::{
    config::{Preferences, ServerConfig},
    database::establish_connection,
    diesel::SaveChangesDsl,
    file_manager::FileManager,
    generic::Generic,
    model::GenericModel,
};
use tracing::{debug, error, info};

use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
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
    pub fn run(&mut self, file_manager: Arc<Mutex<FileManager>>, preferences: &Preferences) {
        info!("Started processing new files");
        file_manager.lock().unwrap().process_new_files(preferences);
        info!("Finished processing new files");
    }
}

///Struct to represent a hashing task. This is needed so we can have an enum
///that contains all types of task.
#[derive(Clone, Debug, Default)]
pub struct Hash {}

impl Hash {
    pub fn run(&self, current_content: Vec<Generic>) -> TaskReturnAsync {
        let is_finished = Arc::new(AtomicBool::new(false));

        info!("Started hashing in the background");
        let is_finished_inner = is_finished.clone();
        //Hash files until all other functions are complete
        let handle = Some(thread::spawn(move || {
            let mut current_content = current_content;
            current_content.retain(|generic| generic.has_hashing_work());
            let length = current_content.len();
            let connection = establish_connection();
            let mut did_finish = true;
            for (i, generic) in current_content.iter_mut().enumerate() {
                for full_path in generic.hash_file_versions() {
                    debug!("Hashed[{} of {}]: {}", i + 1, length, full_path,);
                }
                if GenericModel::from_generic(generic.clone())
                    .save_changes::<GenericModel>(&connection)
                    .is_err()
                {
                    error!("Failed to update hash in database");
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
        file_manager: Arc<Mutex<FileManager>>,
        preferences: &Preferences,
    ) -> Option<TaskReturnAsync> {
        match &mut self.task_type {
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager);
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager, preferences);
            }
            TaskType::Hash(hash) => {
                let mut current_content = file_manager.lock().unwrap().generic_files.clone();
                for show in &file_manager.lock().unwrap().shows {
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
    pub file_manager: Arc<Mutex<FileManager>>,
    pub tasks: Arc<Mutex<VecDeque<Task>>>,
    pub encode_tasks: Arc<Mutex<VecDeque<Task>>>,
    pub config: ServerConfig,
    pub input_completed: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new(
        config: ServerConfig,
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

            let result = task.handle_task(self.file_manager.clone(), preferences);
            if let Some(handle) = result {
                handles.push(handle);
            }
        }
    }
}
