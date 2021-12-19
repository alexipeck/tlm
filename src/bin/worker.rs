use directories::BaseDirs;
use std::thread;
use std::{
    env,
    io::{stdout, Error as IoError},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::JoinHandle,
    time,
};
use tlm::config::WorkerConfig;
use tlm::worker::VersatileMessage;
use tlm::worker_manager::WorkerTranscodeQueue;
use tlm::ws::run_worker;
use tracing::{error, Level};
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

    let file = tracing_appender::rolling::daily(log_path, "tlm_worker.log");
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

    let config = Arc::new(RwLock::new(WorkerConfig::new(
        base_dirs.config_dir().join("tlm/tlm_worker.config"),
    )));

    let worker_uid: Arc<RwLock<Option<i32>>> = Arc::new(RwLock::new(config.read().unwrap().uid));
    let mut handle: Option<JoinHandle<()>> = None;
    loop {
        let transcode_queue: Arc<RwLock<WorkerTranscodeQueue>> =
            Arc::new(RwLock::new(WorkerTranscodeQueue::default()));
        let stop_worker = Arc::new(AtomicBool::new(false));
        let transcode_queue_inner = transcode_queue.clone();
        let stop_worker_inner = stop_worker.clone();
        let (mut tx, rx) = futures_channel::mpsc::unbounded();
        tx.start_send(VersatileMessage::Initialise(config.read().unwrap().uid).to_message())
            .unwrap();

        //TODO: Don't create this thread until we actually have a websocket established
        //Alternatively, don't worry about it, it isn't really a problem as it is currently
        let inner_worker_uid = worker_uid.clone();
        handle = Some(thread::spawn(move || loop {
            transcode_queue
                .write()
                .unwrap()
                .run_transcode(inner_worker_uid.clone(), tx.clone());
            if stop_worker_inner.load(Ordering::Relaxed) {
                break;
            }
        }));

        run_worker(transcode_queue_inner, rx, config.clone()).await?;

        if stop_worker.load(Ordering::Relaxed) {
            break;
        }
        let wait_time = time::Duration::from_secs(1);
        thread::sleep(wait_time);
    }
    if let Some(handle) = handle {
        let _ = handle.join();
    }
    Ok(())
}
