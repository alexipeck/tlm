use crate::{
    database::{
        create_content,
        create_episode,
        establish_connection,
        get_all_content
    },
    content::Content,
    print::{print, From, Verbosity},
    tv::TV,
    utility::Utility,
    scheduler::TaskQueue,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct TrackedDirectories {
    pub root_directories: Vec<String>,
    pub cache_directories: Vec<String>,
}

impl TrackedDirectories {
    pub fn new() -> TrackedDirectories {
        TrackedDirectories {
            root_directories: Vec::new(),
            cache_directories: Vec::new(),
        }
    }
}

pub struct FileManager {
    pub tracked_directories: TrackedDirectories,
    pub working_content: Vec<Content>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub tv: TV,
    pub new_files_queue: Vec<PathBuf>,

    //scheduler
    pub task_queue: TaskQueue,
}

impl FileManager {
    pub fn new(utility: Utility) -> FileManager {
        let mut utility = utility.clone_add_location_start_timing("new(FileManager)", 0);

        let mut file_manager = FileManager {
            tracked_directories: TrackedDirectories::new(),
            tv: TV::new(utility.clone()),
            working_content: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
            task_queue: TaskQueue::new(),
        };

        file_manager.working_content = file_manager.get_all_content(utility.clone());
        file_manager.existing_files_hashset = Content::get_all_filenames_as_hashset_from_content(
            file_manager.working_content.clone(),
            utility.clone(),
        );

        utility.print_function_timer();
        return file_manager;
    }

    pub fn print_number_of_content(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_number_of_content(FileManager)");

        print(
            Verbosity::INFO,
            From::Manager,
            utility,
            format!(
                "Number of content loaded in memory: {}",
                self.working_content.len()
            ),
        );
    }

    pub fn print_number_of_shows(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_number_of_shows(FileManager)");

        print(
            Verbosity::INFO,
            From::Manager,
            utility,
            format!(
                "Number of shows loaded in memory: {}",
                self.tv.working_shows.len()
            ),
        );
    }

    pub fn get_all_content(&mut self, utility: Utility) -> Vec<Content> {
        let mut utility = utility.clone_add_location_start_timing("get_all_contents(Content)", 0);

        let mut content: Vec<Content> = Vec::new();

        let raw_content = get_all_content(utility.clone());

        for content_model in raw_content {
            content.push(Content::from_content_model(
                content_model,
                &mut self.tv.working_shows,
                utility.clone(),
            ));
        }

        utility.print_function_timer();
        return content;
    }

    pub fn process_new_files(&mut self, utility: Utility) {
        let mut utility =
            utility.clone_add_location_start_timing("process_new_files(FileManager)", 0);
        let connection = establish_connection();

        utility.add_timer(0, "startup: processing new files", utility.clone());
        while self.new_files_queue.len() > 0 {
            let current = self.new_files_queue.pop();
            if current.is_some() {
                let current = current.unwrap();

                let mut c = Content::new(&current, &mut self.tv.working_shows, utility.clone());
                let content_model = create_content(
                    &connection,
                    String::from(c.full_path.to_str().unwrap()),
                    c.designation as i32,
                );
                c.content_uid = Some(content_model.id as usize);

                if c.content_is_episode() {
                    let c_uid = c.content_uid.unwrap() as i32;
                    let s_uid = c.show_uid.unwrap() as i32;
                    let (season_number_temp, episode_number_temp) =
                        c.show_season_episode.as_ref().unwrap();
                    let season_number = *season_number_temp as i16;
                    let episode_number = episode_number_temp[0] as i16;
                    create_episode(
                        &connection,
                        c_uid,
                        s_uid,
                        c.show_title.as_ref().unwrap().to_string(),
                        season_number as i32,
                        episode_number as i32,
                    );
                }

                self.working_content.push(c);
            }
        }

        utility.print_function_timer();
    }

    //Hash set guarentees no duplicates in O(1) time
    pub fn import_files(
        &mut self,
        allowed_extensions: &Vec<String>,
        ignored_paths: &Vec<String>,
        utility: Utility,
    ) {
        let mut utility = utility.clone_add_location_start_timing("import_files(FileManager)", 0);

        //Return true if string contains any substring from Vector
        fn str_contains_strs(input_str: &str, substrings: &Vec<String>) -> bool {
            for substring in substrings {
                if String::from(input_str).contains(&substring.to_lowercase()) {
                    return true;
                }
            }
            return false;
        }

        //import all files in tracked root directories
        for directory in &self.tracked_directories.root_directories {
            for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
                if str_contains_strs(
                    &entry.path().to_str().unwrap().to_lowercase(),
                    ignored_paths,
                ) {
                    break;
                }
                if entry.path().is_file() {
                    let temp_string = entry.path().extension().unwrap().to_str().unwrap();
                    if allowed_extensions.contains(&temp_string.to_lowercase()) {
                        let entry_string = entry.clone().into_path();
                        if !self.existing_files_hashset.contains(&entry_string) {
                            self.existing_files_hashset.insert(entry_string.clone());
                            self.new_files_queue.push(entry.clone().into_path());
                        };
                    }
                }
            }
        }

        utility.print_function_timer();
    }
}
