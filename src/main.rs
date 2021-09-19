extern crate diesel;

use tlm::{command_controller::CommandController, config::{Config, Preferences}, scheduler::{Hash, ImportFiles, ProcessNewFiles, Scheduler, Task, TaskType}};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use std::io::stdout;
use tracing::{event, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;

fn main() {
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
    let mut command_controller: CommandController = CommandController::new();
    command_controller.run();
}
