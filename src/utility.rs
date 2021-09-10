use crate::{
    config::Preferences,
    print::{print, From, Verbosity},
    timer::Timer,
};
use std::fmt;

#[derive(Clone, Debug)]
pub enum Traceback {
    ImportFiles,
    ProcessNewFiles,
    PrintTimer,
    RunEncode,
    HandleTask,
    StartScheduler,
    NewConfig,
}

impl Traceback {
    pub fn to_string(&self) -> String {
        match self {
            ImportFiles => {return String::from("import_files(FileManager)")},
            ProcessNewFiles => {return String::from("process_new_files(FileManager)")},
            PrintTimer => {return String::from("print_timer(Timer)")},
            RunEncode => {return String::from("run(Encode)")},
            HandleTask => {return String::from("handle_task(Task)")},
            StartScheduler => {return String::from("start_scheduler(Scheduler)")},
            NewConfig => {return String::from("new(Config)")},
        }
    }
}

#[derive(Clone, Debug)]
pub struct Utility {
    pub traceback: Vec<Traceback>,
    pub current_location: Traceback,
    pub function_timer: Option<Timer>,
    pub preferences: Preferences,
}

impl Utility {
    pub fn new(created_from: Traceback) -> Self {
        let mut utility = Utility {
            traceback: Vec::new(),
            current_location: created_from,
            function_timer: None,
            preferences: Preferences::default(),
        };
        utility.add_traceback_location(created_from)
    }

    pub fn start_function_timer(&mut self) {
        self.function_timer = Some(Timer::create_timer(0, self.current_location.clone()));
    }

    pub fn print_function_timer(&mut self) {
        if !self.preferences.timing_enabled {
            return;
        }
        if self.function_timer.is_some() {
            //the function interally saves inside, but because of the clone, it isn't persistent
            self.function_timer
                .clone()
                .unwrap()
                .print_timer(self.clone());
        } else {
            print(
                Verbosity::CRITICAL,
                From::Utility,
                "You tried to print a timer that doesn't exist.".to_string(),
                false,
                self.clone(),
            );
            panic!()
        }
    }

    fn add_traceback_location(&mut self, called_from: Traceback) -> Utility {
        self.traceback.push(called_from);
        self.clone()
    }

    pub fn clone_add_location(&self, called_from: Traceback) -> Utility {
        let mut temp = self.clone();
        temp.add_traceback_location(called_from);
        temp.current_location = called_from;
        if self.preferences.timing_enabled {
            temp.start_function_timer();
        }
        temp
    }
}

impl fmt::Display for Utility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut call_functions_string: String = String::new();
        let mut single_execute_done = false;
        for function in &self.traceback {
            if !single_execute_done {
                call_functions_string += &format!("'{}'", function.to_string());
                single_execute_done = true;
            } else {
                call_functions_string += &format!(">'{}'", function.to_string());
            }
        }
        write!(f, "{}", call_functions_string)
    }
}
