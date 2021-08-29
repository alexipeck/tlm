use crate::{
    content::Content,
    manager::FileManager,
    print::{print, From, Verbosity},
    utility::Utility,
};
use rand::Rng;
use std::{collections::VecDeque, path::PathBuf, process::Command, sync::atomic::{AtomicUsize, Ordering}};

static TASK_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

    pub fn run(&self, utility: Utility) -> bool {
        let utility = utility.clone_add_location("run(Encode)");
        if !self.is_ready_to_encode() {
            print(
                Verbosity::ERROR,
                From::Scheduler,
                format!("Encode didn't have the required fields for being sent to the encoder"),
                false,
                utility.clone(),
            );
            return false;
        }

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
        return true;
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
    copy: Option<Copy>,
    move_file: Option<MoveFile>,
    rename: Option<Rename>,
    reserve: Option<Reserve>,
    delete: Option<Delete>,
    reencode: Option<Reencode>,
    duplicate: Option<Duplicate>,
    test: Option<Test>,
}

impl Task {
    pub fn new() -> Self {
        return Task {
            task_uid: TASK_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            encode: None,
            copy: None,
            move_file: None,
            rename: None,
            reserve: None,
            delete: None,
            reencode: None,
            duplicate: None,
            test: None,
        };
    }

    pub fn fill_encode(&mut self) {}
    pub fn fill_copy(&mut self) {}
    pub fn fill_move_file(&mut self) {}
    pub fn fill_rename(&mut self) {}
    pub fn fill_reserve(&mut self) {}
    pub fn fill_delete(&mut self) {}
    pub fn fill_reencode(&mut self) {}
    pub fn fill_duplicate(&mut self) {}

    pub fn fill_test(&mut self, test_string: &str) {
        self.test = Some(Test::new(test_string));
    }

    pub fn handle_print_of_task(&mut self, utility: Utility) {
        let utility = utility.clone_add_location("handle_print_of_task(Task)");

        if self.encode.is_some() {
        } else if self.copy.is_some() {
        } else if self.move_file.is_some() {
        } else if self.rename.is_some() {
        } else if self.reserve.is_some() {
        } else if self.delete.is_some() {
        } else if self.reencode.is_some() {
        } else if self.duplicate.is_some() {
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

pub struct TaskQueue {
    tasks: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        return TaskQueue {
            tasks: VecDeque::new(),
        };
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
            task.handle_print_of_task(utility.clone());
        }

        self.tasks = VecDeque::new(); //eh, I can't remember how to check an element and remove it from a Vec or VecDeque
    }
}

pub fn start_scheduler(file_manager: &mut FileManager, utility: Utility) {
    let utility = utility.clone_add_location("start_scheduler");

    let mut rng = rand::thread_rng();
    let mut left: usize = 20;
    let mut iteration_counter: usize = 0;

    loop {
        file_manager.task_queue.handle_tasks(utility.clone());
        if left > 0 {
            let amount_to_add = rng.gen_range(0..5);
            for i in 0..amount_to_add {
                if left > 0 {
                    file_manager.task_queue.push_test_task(&format!(
                        "Task added: {} of {} in iteration {}, left: {}",
                        i + 1,
                        amount_to_add,
                        iteration_counter + 1,
                        left - 1
                    ));
                    left -= 1;
                    iteration_counter += 1;
                }
            }
        } else {
            break;
        }
    }
}
