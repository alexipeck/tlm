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
}

pub enum TaskID {
    Encode,
    Copy,
    MoveFile,
    Rename,
    Reserve,
    Delete,
    Reencode,
    Duplicate,
    Test,
}

#[derive(Clone, Debug)]
//only one should ever be Some()
pub struct Task {
    task_uid: usize,
    encode: Option<Encode>,
    import_files: Option<ImportFiles>,
    process_new_files: Option<ProcessNewFiles>,
    test: Option<Test>,
}

impl Task {
    pub fn new() -> Self {
        return Task {
            task_uid: TASK_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            encode: None,
            import_files: None,
            process_new_files: None,
            test: None,
        };
    }

    pub fn fill_encode(
        &mut self,
        source_path: PathBuf,
        encode_path: PathBuf,
        encode_string: Vec<String>,
    ) {
        self.encode = Some(Encode::new(source_path, encode_path, encode_string));
    }

    pub fn fill_import_files(
        &mut self,
        allowed_extensions: Vec<String>,
        ignored_paths: Vec<String>,
    ) {
        self.import_files = Some(ImportFiles::new(allowed_extensions, ignored_paths));
    }

    pub fn fill_process_files(&mut self) {
        self.process_new_files = Some(ProcessNewFiles::new());
    }

    pub fn fill_test(&mut self, test_string: &str) {
        self.test = Some(Test::new(test_string));
    }

    pub fn handle_task(&mut self, file_manager: &mut FileManager, utility: Utility) {
        let utility = utility.clone_add_location("handle_task(Task)");

        if self.encode.is_some() {
        } else if self.import_files.is_some() {
            let mut import_file_task = self.import_files.clone().unwrap();
            import_file_task.run(file_manager, utility.clone());
        } else if self.process_new_files.is_some() {
            let mut process_new_files_task = self.process_new_files.clone().unwrap();
            process_new_files_task.run(file_manager, utility.clone());
        } else if self.test.is_some() {
            let test = self.test.clone().unwrap();
            print(
                Verbosity::INFO,
                From::Scheduler,
                test.test_string,
                false,
                utility.clone(),
            );
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

    pub fn push_process_new_files_task(&mut self) {
        let mut task = Task::new();
        task.fill_process_files();
        self.tasks.push_back(task);
    }

    pub fn push_import_files_task(
        &mut self,
        allowed_extensions: Vec<String>,
        ignored_paths: Vec<String>,
    ) {
        let mut task = Task::new();
        task.fill_import_files(allowed_extensions, ignored_paths);
        self.tasks.push_back(task);
    }

    pub fn push_test_task(&mut self, test_string: &str) {
        let mut task = Task::new();
        task.fill_test(test_string);
        self.tasks.push_back(task);
    }

    pub fn handle_tasks(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("handle_tasks(TaskQueue)");

        //needs to be safer, but for now it's fine
        for task in &mut self.tasks {
            task.handle_task(&mut self.file_manager, utility.clone());
        }

        self.tasks = VecDeque::new(); //eh, I can't remember how to check an element and remove it from a Vec or VecDeque
    }

    pub fn start_scheduler(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("start_scheduler");

        loop {
            self.handle_tasks(utility.clone());
            break;
        }
    }
}
