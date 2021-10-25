extern crate diesel;

use directories::BaseDirs;
use tlm::{
    config::{Config, Preferences},
    scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType},
    ws::run,
};

use std::collections::VecDeque;
use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::stdout;
use tracing::{event, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::registry::Registry;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let base_dirs = BaseDirs::new().expect("Home directory could not be found");
    let log_path = base_dirs.config_dir().join("tlm/logs/");

    let file = tracing_appender::rolling::daily(log_path, "tlm.log");
    let (stdout_writer, _guard) = tracing_appender::non_blocking(stdout());
    let (file_writer, _guard) = tracing_appender::non_blocking(file);

    let level_filter = LevelFilter::from_level(Level::INFO);
    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(stdout_writer).with_filter(level_filter);

    let logfile_layer = tracing_subscriber::fmt::layer().with_writer(file_writer);

    let subscriber = Registry::default().with(stdout_layer).with(logfile_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    event!(Level::INFO, "Starting tlm");

    let preferences = Preferences::default();

    let config: Config = Config::new(&preferences);

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler =
        Scheduler::new(config.clone(), tasks.clone(), stop_scheduler.clone());

    let inner_pref = preferences.clone();
    //Start the scheduler in it's own thread and return the scheduler at the end
    //so that we can print information before exiting
    let scheduler_handle = thread::spawn(move || {
        scheduler.start_scheduler(&inner_pref);
        scheduler
    });

    //Initial setup in own scope so lock drops
    {
        let mut tasks_guard = tasks.lock().unwrap(); //TODO: Switch to a fair mutex implementation
        tasks_guard.push_back(Task::new(TaskType::Hash(Hash::default())));
        tasks_guard.push_back(Task::new(TaskType::ImportFiles(ImportFiles::default())));
        tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(ProcessNewFiles::default())));
    }

    if !preferences.disable_input {
        run(config.port, tasks).await?;
    }

    stop_scheduler.store(true, Ordering::Relaxed);

    let scheduler = scheduler_handle.join().unwrap();

    scheduler.file_manager.print_number_of_generics();
    scheduler.file_manager.print_number_of_shows();
    scheduler.file_manager.print_number_of_episodes();
    scheduler.file_manager.print_shows(&preferences);
    scheduler.file_manager.print_generics(&preferences);
    scheduler.file_manager.print_episodes(&preferences);
    //scheduler.file_manager.print_rejected_files(); //I'm all for it as soon as it's disabled by default
    Ok(())
}
