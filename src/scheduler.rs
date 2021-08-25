use std::{collections::VecDeque, str::Utf8Error};
use crate::{
    manager::FileManager,
    print::{print, From, Verbosity},
    utility::Utility,
};

#[derive(Clone, Debug)]
pub struct Encode {
    placeholder: Option<String>,
}

impl Encode {
    pub fn new() -> Encode {
        return Encode {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Copy {
    placeholder: Option<String>,
}

impl Copy {
    pub fn new() -> Self {
        return Copy {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MoveFile {
    placeholder: Option<String>,
}

impl MoveFile {
    pub fn new() -> Self {
        return MoveFile {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rename {
    placeholder: Option<String>,
}

impl Rename {
    pub fn new() -> Self {
        return Rename {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reserve {
    placeholder: Option<String>,
}

impl Reserve {
    pub fn new() -> Self {
        return Reserve {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Delete {
    placeholder: Option<String>,
}

impl Delete {
    pub fn new() -> Self {
        return Delete {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reencode {
    placeholder: Option<String>,
}

impl Reencode {
    pub fn new() -> Self {
        return Reencode {
            placeholder: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Duplicate {
    placeholder: Option<String>,
}

impl Duplicate {
    pub fn new() -> Self {
        return Duplicate {
            placeholder: None,
        }
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
        }
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
        let mut task = Task {
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

        /* match task_id {
            TaskID::Encode => task.encode = Some(Encode::new()),
            TaskID::Copy => task.copy = Some(Copy::new()),
            TaskID::MoveFile => task.move_file = Some(MoveFile::new()),
            TaskID::Rename => task.rename = Some(Rename::new()),
            TaskID::Reserve => task.reserve = Some(Reserve::new()),
            TaskID::Delete => task.delete = Some(Delete::new()),
            TaskID::Reencode => task.reencode = Some(Reencode::new()),
            TaskID::Duplicate => task.duplicate = Some(Duplicate::new()),
            TaskID::Test => task.test = Some(Test::new(test_string)),
        } */

        return task;
    }

    pub fn fill_encode(&mut self) {

    }
    pub fn fill_copy(&mut self) {
        
    }
    pub fn fill_move_file(&mut self) {
        
    }
    pub fn fill_rename(&mut self) {
        
    }
    pub fn fill_reserve(&mut self) {
        
    }
    pub fn fill_delete(&mut self) {
        
    }
    pub fn fill_reencode(&mut self) {
        
    }
    pub fn fill_duplicate(&mut self) {
        
    }

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
            print(Verbosity::INFO, From::Scheduler, utility.clone(), test.test_string);
        }
    }
}

pub struct TaskQueue {
    tasks: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        return TaskQueue {
            tasks: VecDeque::new(),
        }
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
    }
}

pub fn start_scheduler(file_manager: &mut FileManager, utility: Utility) {
    let utility = utility.clone_add_location("start_scheduler");
    loop {
        file_manager.task_queue.handle_tasks(utility.clone());
        break;
    }
}
