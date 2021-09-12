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
impl fmt::Display for Traceback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted: &str = match self {
            //FileManager
            Self::AddExistingFilesToHashsetFileManager => {
                "add_existing_files_to_hashset(FileManager)"
            }
            Self::AddAllFilenamesToHashsetFileManager => {
                "add_all_filenames_to_hashset_from_generics(FileManager)"
            }
            Self::PrintNumberOfGenericsFileManager => "print_number_of_generics(FileManager)",
            Self::PrintNumberOfShowsFileManager => "print_number_of_shows(FileManager)",
            Self::PrintNumberOfEpisodesFileManager => "print_number_of_episodes(FileManager)",
            Self::ImportFilesFileManager => "import_files(FileManager)",
            Self::ProcessNewFilesFileManager => "process_new_files(FileManager)",
            Self::PrintEpisodesFileManager => "print_episodes(FileManager)",
            Self::InsertEpisodesFileManager => "insert_episodes(FileManager)",
            Self::EnsureShowExistsFileManager => "ensure_show_exists(FileManager)",
            Self::PrintShowsFileManager => "print_shows(FileManager)",

            //database
            Self::GetAllGenericDatabase => "get_all_generic(database)",
            Self::GetAllShowsDatabase => "get_all_shows(database)",

            //Episode
            Self::PrintEpisodeEpisode => "print_episode(Episode)",

            //Show
            Self::InsertEpisodeShow => "insert_episode(Show)",
            Self::PrintShowShow => "print_show(Show)",
            Self::ShowExistsShow => "show_exists(Show)",
            Self::FromShowModelShow => "from_show_model(Show)",

            //Generic
            Self::NewGeneric => "new(Generic)",
            Self::FromGenericModelGeneric => "from_generic_model(Generic)",
            Self::PrintGenericGeneric => "print_generic(Generic)",
            Self::PrintGenericsGeneric => "print_generics(Generic)",

            //Print
            Self::PrintPrint => "print(print)",

            //_
            Self::Main => "main",
            Self::PrintTimer => "print_timer(Timer)",
            Self::RunEncode => "run(Encode)",
            Self::HandleTask => "handle_task(Task)",
            Self::StartScheduler => "start_scheduler(Scheduler)",
            Self::NewConfig => "new(Config)",
            Self::NewFileManager => "new(FileManager)",
        };
        write!(f, "{}", formatted)
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
