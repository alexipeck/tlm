use crate::{config::{Config, Preferences}, database::establish_connection, diesel::SaveChangesDsl, generic::Generic, manager::FileManager, model::GenericModel};
use tracing::{event, Level};
use websocket::header::Prefer;

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

#[derive(Clone, Debug)]
pub struct ImportFiles {
    allowed_extensions: Vec<String>,
    ignored_paths: Vec<String>,

    pub status_underway: bool,
    pub status_completed: bool,
}

impl ImportFiles {
    pub fn new(allowed_extensions: &[String], ignored_paths: &[String]) -> Self {
        ImportFiles {
            allowed_extensions: allowed_extensions.to_owned(),
            ignored_paths: ignored_paths.to_owned(),
            status_underway: false,
            status_completed: false,
        }
    }

    pub fn run(&mut self, file_manager: &mut FileManager) {
        self.status_underway = true;
        file_manager.import_files();
        self.status_completed = true;
    }
}

#[derive(Clone, Debug)]
pub struct ProcessNewFiles {
    pub status_underway: bool,
    pub status_completed: bool,
}

impl ProcessNewFiles {
    pub fn run(&mut self, file_manager: &mut FileManager, preferences: &Preferences) {
        event!(Level::INFO, "Started processing new files");
        self.status_underway = true;
        file_manager.process_new_files(preferences);
        self.status_completed = true;
        event!(Level::INFO, "Finished processing new files you can now stop the program with Ctrl-c");
    }
    pub fn new() -> Self {
        ProcessNewFiles {
            status_underway: false,
            status_completed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Encode {
    pub source_path: PathBuf,
    pub encode_path: PathBuf,
    pub encode_string: Vec<String>,

    pub status_underway: bool,
    pub status_completed: bool,
}

impl Encode {
    pub fn new(source_path: PathBuf, encode_path: PathBuf, encode_string: Vec<String>) -> Self {
        Encode {
            source_path,
            encode_path,
            encode_string,

            status_underway: false,
            status_completed: false,
        }
    }

    pub fn is_ready_to_encode(&self) -> bool {
        //TODO: Add check for whether the file is ready to go for encode
        true
    }

    pub fn run(&mut self) {
        if !self.is_ready_to_encode() {
            event!(
                Level::WARN,
                "Encode didn't have the required fields for being sent to the encoder"
            );
            return;
        }

        self.status_underway = true;
        event!(
            Level::INFO,
            "Encoding file \'{}\'",
            Generic::get_filename_from_pathbuf(self.source_path.clone())
        );

        let _buffer;
        //linux & friends
        _buffer = Command::new("ffmpeg")
            .args(&self.encode_string.clone())
            .output()
            .expect("failed to execute process");

        //only uncomment if you want disgusting output
        //should be error, but from ffmpeg, stderr mostly consists of stdout information
        //print(Verbosity::DEBUG, "generic", "encode", format!("{}", String::from_utf8_lossy(&buffer.stderr).to_string()));
        self.status_completed = true;
    }
}

pub struct Hash {}

impl Hash {
    pub fn run(&self, current_content: Vec<Generic>) -> TaskReturnAsync {
        let is_finished = Arc::new(AtomicBool::new(false));

        event!(Level::INFO, "Started hashing in the background");
        let is_finished_inner = is_finished.clone();
        //Hash files until all other functions are complete
        let handle = Some(thread::spawn(move || {
            let connection = establish_connection();
            let mut did_finish = true;
            for mut content in current_content {
                if content.hash.is_none() {
                    content.hash();
                    content.fast_hash();
                    if GenericModel::from_generic(content)
                        .save_changes::<GenericModel>(&connection)
                        .is_err()
                    {
                        eprintln!("Failed to update hash in database");
                    }
                }
                if is_finished_inner.load(Ordering::Relaxed) {
                    did_finish = false;
                    break;
                }
            }
            is_finished_inner.store(true, Ordering::Relaxed);
            if did_finish {
                event!(Level::INFO, "Finished hashing");
            } else {
                event!(Level::INFO, "Stopped hashing (incomplete)");
            }
        }))
        .unwrap();

        TaskReturnAsync::new(Some(handle), is_finished)
    }
}

impl Hash {
    pub fn new() -> Self {
        Hash {}
    }
}

pub enum TaskType {
    Encode(Encode),
    ImportFiles(ImportFiles),
    ProcessNewFiles(ProcessNewFiles),
    Hash(Hash),
}

#[allow(dead_code)]
pub struct Task {
    task_uid: usize,
    task_type: TaskType,
}

impl Task {
    pub fn new(task_type: TaskType) -> Self {
        Task {
            task_uid: TASK_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            task_type,
        }
    }

    pub fn handle_task(
        &mut self,
        file_manager: &mut FileManager,
        preferences: &Preferences,
    ) -> Option<TaskReturnAsync> {
        match &mut self.task_type {
            TaskType::Encode(encode) => {
                encode.run();
            }
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager);
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager, &preferences);
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

pub struct TaskReturnAsync {
    handle: Option<JoinHandle<()>>,
    is_done: Arc<AtomicBool>,
}

impl TaskReturnAsync {
    pub fn new(handle: Option<JoinHandle<()>>, is_done: Arc<AtomicBool>) -> Self {
        Self { handle, is_done }
    }
}

pub struct Scheduler {
    pub file_manager: FileManager,
    pub tasks: Arc<Mutex<VecDeque<Task>>>,
    pub config: Config,
    pub input_completed: Arc<AtomicBool>,
}

impl Scheduler {
    pub fn new(
        config: Config,
        tasks: Arc<Mutex<VecDeque<Task>>>,
        input_completed: Arc<AtomicBool>,
    ) -> Self {
        Scheduler {
            tasks,
            file_manager: FileManager::new(&config),
            config,
            input_completed,
        }
    }

    pub fn start_scheduler(&mut self, preferences: &Preferences) {
        let wait_time = time::Duration::from_secs(1);

        //Take a handle from any async function and 2 Bools
        //The Handle is in an option so we can take the Handle in order to join it,
        //that is neccesary because otherwise it is owned by the vector and joining would destroy it
        //The first bool tells the thread to stop.
        //The second bool tells us that the thread is complete
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
                let _res = handle.unwrap().join();
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
                    std::mem::drop(tasks); //Unlock the mutex so we don't block while sleeping
                    thread::sleep(wait_time);
                    continue;
                }
                task = tasks.pop_front().unwrap();
            }

            let result = task.handle_task(&mut self.file_manager, &preferences);
            if let Some(handle) = result {
                handles.push(handle)
            }
        }
    }
}
