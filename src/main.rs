extern crate diesel;

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
use tracing_subscriber::registry::Registry;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    //Optimal seems to be 2x the number of threads but more testing required
    //By default this is the number of threads the cpu has
    //rayon::ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();
    let file = tracing_appender::rolling::daily("./logs", "tlm.log");
    let (writer, _guard) = tracing_appender::non_blocking(stdout());
    let (writer2, _guard) = tracing_appender::non_blocking(file);
    let layer = tracing_subscriber::fmt::layer().with_writer(writer);

    let layer2 = tracing_subscriber::fmt::layer().with_writer(writer2);

    let subscriber = Registry::default().with(layer).with(layer2);

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

        tasks_guard.push_back(Task::new(TaskType::ProcessNewFiles(
            ProcessNewFiles::default(),
        )));
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
