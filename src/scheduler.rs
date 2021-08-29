use crate::{
    config::Config,
    content::Content,
    manager::FileManager,
    print::{print, From, Verbosity},
    utility::Utility,
};
use std::{
    collections::VecDeque,
    fs::File,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
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
    pub fn new(allowed_extensions: Vec<String>, ignored_paths: Vec<String>) -> Self {
        return ImportFiles {
            allowed_extensions: allowed_extensions,
            ignored_paths: ignored_paths,
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

        let buffer;
        if !cfg!(target_os = "windows") {
            //linux & friends
            buffer = Command::new("ffmpeg")
                .args(&self.encode_string.clone())
                .output()
                .expect("failed to execute process");
        } else {
            //windows
            buffer = Command::new("ffmpeg")
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
pub struct Copy {
    placeholder: Option<String>,
}

impl Copy {
    pub fn new() -> Self {
        return Copy { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct MoveFile {
    placeholder: Option<String>,
}

impl MoveFile {
    pub fn new() -> Self {
        return MoveFile { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct Rename {
    placeholder: Option<String>,
}

impl Rename {
    pub fn new() -> Self {
        return Rename { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct Reserve {
    placeholder: Option<String>,
}

impl Reserve {
    pub fn new() -> Self {
        return Reserve { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct Delete {
    placeholder: Option<String>,
}

impl Delete {
    pub fn new() -> Self {
        return Delete { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct Reencode {
    placeholder: Option<String>,
}

impl Reencode {
    pub fn new() -> Self {
        return Reencode { placeholder: None };
    }
}

#[derive(Clone, Debug)]
pub struct Duplicate {
    placeholder: Option<String>,
}

impl Duplicate {
    pub fn new() -> Self {
        return Duplicate { placeholder: None };
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
        print(
            Verbosity::INFO,
            From::Scheduler,
            format!("Test was {}", self.test_string),
            false,
            utility.clone(),
        );
    }
}

pub enum TaskType {
    Encode(Encode),
    ImportFiles(ImportFiles),
    ProcessNewFiles(ProcessNewFiles),
    Test(Test),
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

    pub fn handle_task(&mut self, file_manager: &mut FileManager, utility: Utility) {
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
        }
    }
}

pub struct Scheduler {
    pub file_manager: FileManager,
    pub tasks: VecDeque<Task>,
}

impl Scheduler {
    pub fn new(config: &Config, utility: Utility) -> Self {
        return Scheduler {
            tasks: VecDeque::new(),
            file_manager: FileManager::new(&config, utility),
        };
    }

    pub fn start_scheduler(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("start_scheduler");

        self.tasks
            .push_back(Task::new(TaskType::Test(Test::new("first"))));

        self.tasks
            .push_back(Task::new(TaskType::Test(Test::new("second"))));

        self.tasks
            .push_back(Task::new(TaskType::Test(Test::new("thirds"))));

        loop {
            if self.tasks.len() == 0 {
                break;
            }
            let mut task = self.tasks.pop_front().unwrap();
            task.handle_task(&mut self.file_manager, utility.clone());
        }
    }
}
