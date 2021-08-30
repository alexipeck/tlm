extern crate diesel;
use diesel::query_dsl::SaveChangesDsl;
use tlm::{
    config::Config,
    database::establish_connection,
    model::ContentModel,
    scheduler::{ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType, Test},
    utility::Utility,
};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use text_io::read;

fn main() {
    //traceback and timing utility
    let utility = Utility::new("main");

    let config: Config = Config::new(&utility.preferences);

    //Needs to be moved to scheduler task
    /*let original_files = scheduler.file_manager.working_content.clone();

    let stop_background = Arc::new(AtomicBool::new(false));
    let stop_background_inner = stop_background.clone();

    //Hash files until all other functions are complete
    let hash_handle = thread::spawn(move || {
        let connection = establish_connection();
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
    });*/

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    //scheduler.push_import_files_task();
    //scheduler.push_process_new_files_task();
    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler = Scheduler::new(
        config.clone(),
        utility.clone(),
        tasks.clone(),
        stop_scheduler.clone(),
    );
    let utility_inner = utility.clone();
    let scheduler_handle = thread::spawn(move || {
        scheduler.start_scheduler(utility_inner.clone());
        scheduler
    });

    //Initial setup in own scope so lock drops
    {
        let mut tasks_guard = tasks.lock().unwrap();
        tasks_guard.push_back(Task::new(TaskType::ImportFiles(ImportFiles::new(
            &config.allowed_extensions,
            &config.ignored_paths,
        ))));

        tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(ProcessNewFiles::new())));
    }

    //Placeholder user input
    println!("Pick a number to print or -1 to stop");
    loop {
        let input: i32 = read!();
        if input == -1 {
            break;
        }

        {
            let mut tasks_guard = tasks.lock().unwrap();
            tasks_guard.push_back(Task::new(TaskType::Test(Test::new(&format!(
                "Entered: {}",
                input
            )))));
        }
    }

    stop_scheduler.store(true, Ordering::Relaxed);
    let scheduler = scheduler_handle.join().unwrap();

    scheduler
        .file_manager
        .print_number_of_content(utility.clone());
    scheduler
        .file_manager
        .print_number_of_shows(utility.clone());

    scheduler.file_manager.print_shows(utility.clone());
    scheduler.file_manager.print_content(utility.clone());

    //Tell worker thread to stop after it has finished hashing current file
    //stop_background.store(true, Ordering::Relaxed);
    //let _res = hash_handle.join();
}
