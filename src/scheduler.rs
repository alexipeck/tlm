use crate::diesel::SaveChangesDsl;
use crate::{
    config::Config,
    content::Content,
    database::establish_connection,
    manager::FileManager,
    model::ContentModel,
    print::{print, From, Verbosity},
    utility::Utility,
};

use std::{
    collections::VecDeque,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    thread, time,
};
use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::JoinHandle,
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
    pub fn new(allowed_extensions: &Vec<String>, ignored_paths: &Vec<String>) -> Self {
        ImportFiles {
            allowed_extensions: allowed_extensions.clone(),
            ignored_paths: ignored_paths.clone(),
            status_underway: false,
            status_completed: false,
        }
    }

    pub fn run(&mut self, file_manager: &mut FileManager, utility: Utility) {
        self.status_underway = true;
        file_manager.import_files(&self.allowed_extensions, &self.ignored_paths, utility);
        self.status_completed = true;
    }
}

#[derive(Clone, Debug)]
pub struct ProcessNewFiles {
    pub status_underway: bool,
    pub status_completed: bool,
}

impl ProcessNewFiles {
    pub fn run(&mut self, file_manager: &mut FileManager, utility: Utility) {
        self.status_underway = true;
        file_manager.process_new_files(utility);
        self.status_completed = true;
    }
}

impl Default for ProcessNewFiles {
    fn default() -> Self {
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

    pub fn run(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("run(Encode)");
        if !self.is_ready_to_encode() {
            print(
                Verbosity::ERROR,
                From::Scheduler,
                "Encode didn't have the required fields for being sent to the encoder".to_string(),
                false,
                utility,
            );
            return;
        }

        self.status_underway = true;

        print(
            Verbosity::INFO,
            From::Job,
            format!(
                "Encoding file \'{}\'",
                Content::get_filename_from_pathbuf(self.source_path.clone())
            ),
            false,
            utility,
        );

        let _buffer;
        //linux & friends
        _buffer = Command::new("ffmpeg")
            .args(&self.encode_string.clone())
            .output()
            .expect("failed to execute process");

        //only uncomment if you want disgusting output
        //should be error, but from ffmpeg, stderr mostly consists of stdout information
        //print(Verbosity::DEBUG, "content", "encode", format!("{}", String::from_utf8_lossy(&buffer.stderr).to_string()));
        self.status_completed = true;
    }
}

#[derive(Clone, Debug)]
pub struct Test {
    test_string: String,
}

impl Test {
    pub fn new(test_string: &str) -> Self {
        Test {
            test_string: String::from(test_string),
        }
    }

    pub fn run(&self, utility: Utility) {
        let utility = utility.clone_add_location("run(Test)");
        let wait_time = time::Duration::from_secs(2); //Just to illustrate threadedness
        thread::sleep(wait_time);
        print(
            Verbosity::INFO,
            From::Scheduler,
            self.test_string.to_string(),
            false,
            utility,
        );
    }
}

pub struct Hash {}

impl Hash {
    pub fn run(&mut self, current_content: Vec<Content>) -> TaskReturnAsync {
        let stop_hash = Arc::new(AtomicBool::new(false));
        let is_finished = Arc::new(AtomicBool::new(false));

        let stop_hash_inner = stop_hash.clone();
        let is_finished_inner = is_finished.clone();
        //Hash files until all other functions are complete
        let handle = Some(thread::spawn(move || {
            let connection = establish_connection();
            for mut content in current_content {
                if content.hash.is_none() {
                    content.hash();
                    if ContentModel::from_content(content)
                        .save_changes::<ContentModel>(&connection)
                        .is_err()
                    {
                        eprintln!("Failed to update hash in database");
                    }
                }
                if stop_hash_inner.load(Ordering::Relaxed) {
                    break;
                }
            }
            is_finished_inner.store(true, Ordering::Relaxed);
        }))
        .unwrap();
        TaskReturnAsync::new(Some(handle), stop_hash, is_finished)
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash {}
    }
}

pub enum TaskType {
    Encode(Encode),
    ImportFiles(ImportFiles),
    ProcessNewFiles(ProcessNewFiles),
    Test(Test),
    Hash(Hash),
}

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
        utility: Utility,
    ) -> Option<TaskReturnAsync> {
        let utility = utility.clone_add_location("handle_task(Task)");

        match &mut self.task_type {
            TaskType::Encode(encode) => {
                encode.run(utility);
            }
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager, utility);
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager, utility);
            }
            TaskType::Test(test) => {
                test.run(utility);
            }
            TaskType::Hash(hash) => {
                return Some(hash.run(file_manager.working_content.clone()));
            }
        }
        None
    }
}

pub struct TaskReturnAsync {
    handle: Option<JoinHandle<()>>,
    send_wrapup: Arc<AtomicBool>,
    is_done: Arc<AtomicBool>,
}

impl TaskReturnAsync {
    pub fn new(
        handle: Option<JoinHandle<()>>,
        send_wrapup: Arc<AtomicBool>,
        is_done: Arc<AtomicBool>,
    ) -> Self {
        Self {
            handle,
            send_wrapup,
            is_done,
        }
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
        utility: Utility,
        tasks: Arc<Mutex<VecDeque<Task>>>,
        input_completed: Arc<AtomicBool>,
    ) -> Self {
        Scheduler {
            tasks,
            file_manager: FileManager::new(&config, utility),
            config,
            input_completed,
        }
    }

    pub fn start_scheduler(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("start_scheduler");
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
                            handle.send_wrapup.store(true, Ordering::Relaxed);
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

            let result = task.handle_task(&mut self.file_manager, utility.clone());
            if let Some(handle) = result {
                handles.push(handle)
            }
        }
    }
}
