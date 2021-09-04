extern crate diesel;
use tlm::{
    config::Config,
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

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler = Scheduler::new(
        config.clone(),
        utility.clone(),
        tasks.clone(),
        stop_scheduler.clone(),
    );
    let utility_inner = utility.clone();

    //Start the scheduler in it's own thread and return the scheduler at the end
    //so that we can print information before exiting
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
    if !utility.preferences.disable_input {
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
    }

    stop_scheduler.store(true, Ordering::Relaxed);
    let scheduler = scheduler_handle.join().unwrap();

    scheduler
        .file_manager
        .print_number_of_generic(utility.clone());
    scheduler
        .file_manager
        .print_number_of_shows(utility.clone());

    scheduler.file_manager.print_shows(utility.clone());
    scheduler.file_manager.print_generics(utility.clone());
}
