extern crate diesel;
use diesel::query_dsl::SaveChangesDsl;
use tlm::{
    config::{Config, Preferences},
    database::establish_connection,
    manager::FileManager,
    model::ContentModel,
    print::Verbosity,
    scheduler::start_scheduler,
    utility::Utility,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    //traceback and timing utility
    let mut utility = Utility::new("main", 0);

    let config = Config::new(&utility.preferences);

    utility.min_verbosity =
        Verbosity::from_string(&utility.preferences.min_verbosity.to_uppercase());

    //The FileManager stores working files, hashsets and supporting functions related to updating those files
    let mut file_manager: FileManager = FileManager::new(utility.clone());
    let original_files = file_manager.working_content.clone();

    let stop_background = Arc::new(AtomicBool::new(false));
    let stop_background_inner = stop_background.clone();
    let connection = establish_connection();

    //Hash files until all other functions are complete
    let handle = thread::spawn(move || {
        for mut c in original_files {
            if c.hash.is_none() {
                c.hash();
                if ContentModel::from_content(c)
                    .save_changes::<ContentModel>(&connection)
                    .is_err()
                {
                    eprintln!("Failed to update hash in database");
                }
            }
            if stop_background_inner.load(Ordering::Relaxed) {
                break;
            }
        }
    });

    file_manager.tracked_directories = config.tracked_directories;
    file_manager.import_files(
        &config.allowed_extensions,
        &config.ignored_paths,
        utility.clone(),
    );

    file_manager.process_new_files(utility.clone());

    file_manager.print_number_of_content(utility.clone());
    file_manager.print_number_of_shows(utility.clone());

    file_manager.task_queue.push_test_task("Main");
    start_scheduler(&mut file_manager, utility.clone());

    //Tell worker thread to stop after it has finished hashing current file
    stop_background.store(true, Ordering::Relaxed);
    let _res = handle.join();
}
