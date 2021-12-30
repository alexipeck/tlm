extern crate diesel;

use directories::BaseDirs;
use tlm::{
    config::{Preferences, ServerConfig},
    encode::Encode,
    file_manager::FileManager,
    scheduler::{Scheduler, Task},
    worker::Worker,
    worker_manager::WorkerManager,
    ws::run_web,
};

use core::time;
use std::{collections::VecDeque, sync::RwLock};
use std::env;
use std::io::stdout;
use std::io::Error as IoError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{error, info, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::Layer;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let stdout_level = match env::var("TLM_DISPLAYED_LEVEL") {
        Ok(level_str) => match level_str.to_lowercase().as_str() {
            "info" => Some(Level::INFO),
            "debug" => Some(Level::DEBUG),
            "warning" | "warn" => Some(Level::WARN),
            "trace" => Some(Level::TRACE),
            "error" | "err" => Some(Level::ERROR),
            _ => None,
        },
        Err(_) => None,
    };

    let base_dirs = BaseDirs::new().unwrap_or_else(|| {
        error!("Home directory could not be found");
        panic!();
    });
    let log_path = base_dirs.config_dir().join("tlm/logs/");

    let file = tracing_appender::rolling::daily(log_path, "tlm_server");
    let (stdout_writer, _guard) = tracing_appender::non_blocking(stdout());
    let (file_writer, _guard) = tracing_appender::non_blocking(file);

    let level_filter;
    if let Some(level) = stdout_level {
        level_filter = LevelFilter::from_level(level);
    } else {
        level_filter = LevelFilter::from_level(Level::INFO);
    }
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(stdout_writer)
        .with_filter(level_filter);

    let logfile_layer = tracing_subscriber::fmt::layer().with_writer(file_writer);

    let subscriber = Registry::default().with(stdout_layer).with(logfile_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Starting tlm");

    let preferences = Preferences::default();

    let server_config: Arc<RwLock<ServerConfig>> = Arc::new(RwLock::new(ServerConfig::new(&preferences)));

    let tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));

    let encode_tasks: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));
    let worker_mananger_workers: Arc<Mutex<VecDeque<Worker>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    let worker_mananger_transcode_queue: Arc<Mutex<VecDeque<Encode>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    //NOTE: Once the establish/reestablish functionality has been separated from the WorkerManager, the worker_manager shouldn't need an Arc<Mutex<>>>
    let worker_manager: Arc<Mutex<WorkerManager>> = Arc::new(Mutex::new(WorkerManager::new(
        worker_mananger_workers,
        worker_mananger_transcode_queue.clone(),
        600,
    )));
    let file_manager: Arc<Mutex<FileManager>> = Arc::new(Mutex::new(FileManager::new(server_config.clone())));

    let stop_scheduler = Arc::new(AtomicBool::new(false));
    let mut scheduler: Scheduler = Scheduler::new(
        server_config.clone(),
        tasks.clone(),
        encode_tasks,
        file_manager.clone(),
        stop_scheduler.clone(),
    );

    //Start the scheduler in it's own thread and return the scheduler at the end
    //so that we can print information before exiting
    let scheduler_handle = thread::spawn(move || {
        scheduler.start_scheduler();
        scheduler
    });

    let stop_worker_mananger_polling = Arc::new(AtomicBool::new(false));
    let inner_stop_worker_manager_polling = stop_worker_mananger_polling.clone();
    let worker_manager_polling_wait_time = time::Duration::from_secs_f64(2.0);
    let inner_worker_manager = worker_manager.clone();
    let worker_manager_polling_handle = thread::spawn(move || {
        while !inner_stop_worker_manager_polling.load(Ordering::Relaxed) {
            inner_worker_manager.lock().unwrap().polling_event();
            thread::sleep(worker_manager_polling_wait_time);
        }
        inner_worker_manager
    });

    let port = server_config.read().unwrap().port;

    if !preferences.disable_input {
        run_web(
            port,
            tasks,
            file_manager,
            worker_mananger_transcode_queue,
            worker_manager,
            server_config.clone()
        )
        .await?;
    }

    stop_scheduler.store(true, Ordering::Relaxed);
    stop_worker_mananger_polling.store(true, Ordering::Relaxed);

    //manual shutdown tasks or other manipulation
    let _scheduler = scheduler_handle.join().unwrap();
    let _worker_manager = worker_manager_polling_handle.join().unwrap();

    Ok(())
}
