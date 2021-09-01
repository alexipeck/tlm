extern crate diesel;
use tlm::{
    config::Config,
    scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType},
    utility::Utility,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    //traceback and timing utility
    let utility = Utility::new("main");
    let progress_bars = MultiProgress::new();
    let import_bar =
        progress_bars.add(ProgressBar::new(0).with_style(ProgressStyle::default_spinner()));
    let process_bar =
        progress_bars.add(ProgressBar::new(0).with_style(ProgressStyle::default_spinner()));
    let hash_bar =
        progress_bars.add(ProgressBar::new(0).with_style(ProgressStyle::default_spinner()));

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
        tasks_guard.push_back(Task::new(TaskType::Hash(Hash::new(hash_bar))));

        tasks_guard.push_back(Task::new(TaskType::ImportFiles(ImportFiles::new(
            &config.allowed_extensions,
            &config.ignored_paths,
            import_bar,
        ))));

        tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(ProcessNewFiles::new(
            process_bar,
        ))));
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
    scheduler.file_manager.print_content(utility);
}
