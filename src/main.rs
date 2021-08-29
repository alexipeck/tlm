extern crate diesel;
use diesel::query_dsl::SaveChangesDsl;
use tlm::{
    config::Config, database::establish_connection, manager::FileManager, model::ContentModel,
    print::Verbosity, scheduler::Scheduler, utility::Utility,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    //traceback and timing utility
    let mut utility: Utility = Utility::new("main", 0);

    let config: Config = Config::new(&utility.preferences);

    let mut scheduler: Scheduler = Scheduler::new(&config, utility.clone());

    utility.min_verbosity =
        Verbosity::from_string(&utility.preferences.min_verbosity.to_uppercase());

    //The FileManager stores working files, hashsets and supporting functions related to updating those files
    let original_files = scheduler.file_manager.working_content.clone();

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

    scheduler.start_scheduler(utility.clone());

    scheduler
        .file_manager
        .print_number_of_content(utility.clone());
    scheduler
        .file_manager
        .print_number_of_shows(utility.clone());

    scheduler.file_manager.print_shows(utility.clone());
    scheduler.file_manager.print_content(utility.clone());

    //Tell worker thread to stop after it has finished hashing current file
    stop_background.store(true, Ordering::Relaxed);
    let _res = handle.join();
}
