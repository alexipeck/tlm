use crate::{
    config::Preferences,
    print::{print, From, Verbosity},
    timer::Timer,
};
use std::fmt;

#[derive(Clone, Debug)]
pub enum Traceback {    
    //FileManager
    AddExistingFilesToHashsetFileManager,
    AddAllFilenamesToHashsetFileManager,
    PrintNumberOfGenericsFileManager,
    PrintNumberOfShowsFileManager,
    PrintNumberOfEpisodesFileManager,
    ImportFilesFileManager,
    ProcessNewFilesFileManager,
    PrintEpisodesFileManager,
    InsertEpisodesFileManager,
    EnsureShowExistsFileManager,
    PrintShowsFileManager,

    //Database
    GetAllGenericDatabase,
    GetAllShowsDatabase,
    
    //Episode
    PrintEpisodeEpisode,        

    //Show
    InsertEpisodeShow,
    PrintShowShow,
    ShowExistsShow,
    FromShowModelShow,

    //Generic
    NewGeneric,
    FromGenericModelGeneric,
    PrintGenericGeneric,
    PrintGenericsGeneric,

    //Print
    PrintPrint,

    //Unsorted
    Main,
    PrintTimer,
    RunEncode,
    HandleTask,
    StartScheduler,
    NewConfig,
    NewFileManager,
}

//lower-case item in parenthasis implies *.rs file rather than a struct
#[allow(clippy::inherent_to_string)]
#[allow(bindings_with_variant_name)]
impl Traceback {
    pub fn to_string(&self) -> String {
        match self {

            //FileManager
            AddExistingFilesToHashsetFileManager => {return String::from("add_existing_files_to_hashset(FileManager)")},
            AddAllFilenamesToHashsetFileManager => {return String::from("add_all_filenames_to_hashset_from_generics(FileManager)")},
            PrintNumberOfGenericsFileManager => {return String::from("print_number_of_generics(FileManager)")},
            PrintNumberOfShowsFileManager => {return String::from("print_number_of_shows(FileManager)")},
            PrintNumberOfEpisodesFileManager => {return String::from("print_number_of_episodes(FileManager)")},
            ImportFiles  => {return String::from("import_files(FileManager)")},
            ProcessNewFiles  => {return String::from("process_new_files(FileManager)")},
            PrintEpisodesFileManager => {return String::from("print_episodes(FileManager)")},
            InsertEpisodesFileManager => {return String::from("insert_episodes(FileManager)")},
            EnsureShowExistsFileManager => {return String::from("ensure_show_exists(FileManager)")},
            PrintShowsFileManager => {return String::from("print_shows(FileManager)")},
            
            //database
            GetAllGenericDatabase => {return String::from("get_all_generic(database)")},
            GetAllShowsDatabase => {return String::from("get_all_shows(database)")},

            //Episode
            InsertEpisodeShow => {return String::from("print_episode(Episode)")},

            //Show
            InsertEpisodeShow => {return String::from("insert_episode(Show)")},
            PrintShowShow => {return String::from("print_show(Show)")},
            ShowExistsShow => {return String::from("show_exists(Show)")},
            FromShowModelShow => {return String::from("from_show_model(Show)")},

            //Generic
            NewGeneric => {return String::from("new(Generic)")},
            FromGenericModelGeneric => {return String::from("from_generic_model(Generic)")},
            PrintGenericGeneric => {return String::from("print_generic(Generic)")},
            PrintGenericsGeneric => {return String::from("print_generics(Generic)")},

            //_
            Main => {return String::from("main")},
            PrintTimer => {return String::from("print_timer(Timer)")},
            RunEncode => {return String::from("run(Encode)")},
            HandleTask => {return String::from("handle_task(Task)")},
            StartScheduler => {return String::from("start_scheduler(Scheduler)")},
            NewConfig => {return String::from("new(Config)")},
            NewFileManager => {return String::from("new(FileManager)")},
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
            current_location: created_from.clone(),
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
        temp.current_location = called_from.clone();
        temp.add_traceback_location(called_from);
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
