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
        return ImportFiles {
            allowed_extensions: allowed_extensions.clone(),
            ignored_paths: ignored_paths.clone(),
            status_underway: false,
            status_completed: false,
        };
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
    pub fn new() -> Self {
        return ProcessNewFiles {
            status_underway: false,
            status_completed: false,
        };
    }

    pub fn run(&mut self, file_manager: &mut FileManager, utility: Utility) {
        self.status_underway = true;
        file_manager.process_new_files(utility);
        self.status_completed = true;
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
        return Encode {
            source_path: source_path,
            encode_path: encode_path,
            encode_string: encode_string,

            status_underway: false,
            status_completed: false,
        };
    }

    pub fn is_ready_to_encode(&self) -> bool {
        //TODO: Add check for whether the file is ready to go for encode
        return true;
    }

    pub fn run(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("run(Encode)");
        if !self.is_ready_to_encode() {
            print(
                Verbosity::ERROR,
                From::Scheduler,
                format!("Encode didn't have the required fields for being sent to the encoder"),
                false,
                utility.clone(),
            );
            return;
        }

        self.status_underway = true;

        print(
            Verbosity::INFO,
            From::Job,
            format!(
                "Encoding file \'{}\'",
                Content::get_filename_from_pathbuf(self.source_path.clone().clone())
            ),
            false,
            utility,
        );

        let _buffer;
        if !cfg!(target_os = "windows") {
            //linux & friends
            _buffer = Command::new("ffmpeg")
                .args(&self.encode_string.clone())
                .output()
                .expect("failed to execute process");
        } else {
            //windows
            _buffer = Command::new("ffmpeg")
                .args(&self.encode_string.clone())
                .output()
                .expect("failed to execute process");
        }
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
        return Test {
            test_string: String::from(test_string),
        };
    }

    pub fn run(&self, utility: Utility) {
        let utility = utility.clone_add_location("run(Test)");
        let wait_time = time::Duration::from_secs(2); //Just to illustrate threadedness
        thread::sleep(wait_time);
        print(
            Verbosity::INFO,
            From::Scheduler,
            format!("{}", self.test_string),
            false,
            utility.clone(),
        );
    }
}

pub struct Hash {}

impl Hash {
    pub fn new() -> Self {
        return Hash {};
    }

    pub fn run(
        &mut self,
        current_content: Vec<Content>,
    ) -> (Option<JoinHandle<()>>, Arc<AtomicBool>, Arc<AtomicBool>) {
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
        }))
        .unwrap();
        return (Some(handle), stop_hash, is_finished);
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
        return Task {
            task_uid: TASK_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            task_type: task_type,
        };
    }

    pub fn handle_task(
        &mut self,
        file_manager: &mut FileManager,
        utility: Utility,
    ) -> Option<(Option<JoinHandle<()>>, Arc<AtomicBool>, Arc<AtomicBool>)> {
        let utility = utility.clone_add_location("handle_task(Task)");

        match &mut self.task_type {
            TaskType::Encode(encode) => {
                encode.run(utility.clone());
            }
            TaskType::ImportFiles(import_files) => {
                import_files.run(file_manager, utility.clone());
            }
            TaskType::ProcessNewFiles(process_new_files) => {
                process_new_files.run(file_manager, utility.clone());
            }
            TaskType::Test(test) => {
                test.run(utility.clone());
            }
            TaskType::Hash(hash) => {
                return Some(hash.run(file_manager.working_content.clone()));
            }
        }
        None
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
        completed_marker: Arc<AtomicBool>,
    ) -> Self {
        return Scheduler {
            tasks: tasks,
            file_manager: FileManager::new(&config, utility),
            config: config,
            input_completed: completed_marker,
        };
    }

    pub fn start_scheduler(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("start_scheduler");
        let wait_time = time::Duration::from_secs(1);

        //Take a handle from any async function and 2 Bools
        //The Handle is in an option so we can take the Handle in order to join it,
        //that is neccesary because otherwise it is owned by the vector and joining would destroy it
        //The first bool tells the thread to stop.
        //The second bool tells us that the thread is complete
        let mut handles: Vec<(Option<JoinHandle<()>>, Arc<AtomicBool>, Arc<AtomicBool>)> =
            Vec::new();

        loop {
            let mut task: Task;

            //Mark the completed threads
            let mut completed_threads: Vec<usize> = Vec::new();
            for (i, opts) in handles.iter().enumerate() {
                if opts.2.load(Ordering::Relaxed) {
                    //opts.0.join();
                    completed_threads.push(i);
                }
            }

            //Reverse so we can remove items right to left avoiding the index shifting
            completed_threads.reverse();

            //Take each completed thread and join it
            for i in completed_threads {
                let handle = handles[i].0.take();
                let _res = handle.unwrap().join();
                handles.remove(i);
            }
            {
                let mut tasks = self.tasks.lock().unwrap();

                //When the queue is empty we wait until another item is added or user input is marked as completed
                if tasks.len() == 0 {
                    if self.input_completed.load(Ordering::Relaxed) {
                        //Tell all async tasks to stop early if they can
                        for opts in handles {
                            opts.1.store(true, Ordering::Relaxed);
                            let _res = opts.0.unwrap().join();
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
            if result.is_some() {
                handles.push(result.unwrap())
            }
        }
    }
}
