use crate::{generic::Generic, profile::Profile};
use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

static WORKER_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

///Struct to represent a file encode task. This is needed so we can have an enum
///that contains all types of task
///This should probably handle it's current variables without having them passed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub source_path: PathBuf,
    pub future_filename: String,
    pub encode_options: Vec<String>,
    pub profile: Profile,
}

impl Encode {
    pub fn new(
        source_path: PathBuf,
        future_filename: String,
        encode_options: Vec<String>,
        profile: Profile,
    ) -> Self {
        Self {
            source_path,
            future_filename,
            encode_options,
            profile,
        }
    }

    pub fn run(&mut self) {
        info!(
            "Encoding file \'{}\'",
            Generic::get_filename_from_pathbuf(self.source_path.clone())
        );

        let _buffer = Command::new("ffmpeg")
            .args(&self.encode_options.clone())
            .output()
            .unwrap_or_else(|err| {
                error!("Failed to execute ffmpeg process. Err: {}", err);
                panic!();
            });

        //only uncomment if you want disgusting output
        //should be error, but from ffmpeg, stderr mostly consists of stdout information
        //print(Verbosity::DEBUG, "generic", "encode", format!("{}", String::from_utf8_lossy(&buffer.stderr).to_string()));
    }
}

#[derive(Debug)] //, Serialize, Deserialize
pub struct Worker {
    uid: usize,
    tx: UnboundedSender<Message>,
    transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
    transcode_queue_capacity: usize,
    //TODO: Time remaining on current episode
    //TODO: Current encode percentage
    //TODO: Worker UID, should be based on some hardware identifier, so it can be regenerated
    //NOTE: If this is running under a Docker container, it may have a random MAC address, so on reboot,
    //    : becoming a new worker will probably mean the old one should be dropped after x amount of time.
}

impl Worker {
    pub fn new(tx: UnboundedSender<Message>, transcode_queue_capacity: usize) -> Self {
        Self {
            uid: WORKER_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            tx,
            transcode_queue: Arc::new(RwLock::new(VecDeque::new())),
            transcode_queue_capacity,
        }
    }

    pub fn spaces_in_queue(&mut self) -> usize {
        self.transcode_queue.read().unwrap().len() - self.transcode_queue_capacity
    }

    pub fn fill_transcode_queue(&mut self, transcode_queue: Arc<Mutex<VecDeque<Encode>>>) {
        if transcode_queue.lock().unwrap().is_empty() {
            return;
        }
        for _ in 0..self.spaces_in_queue() {
            match transcode_queue.lock().unwrap().pop_front() {
                Some(encode) => {
                    self.transcode_queue.write().unwrap().push_back(encode);
                }
                None => {
                    info!("No encode tasks to send to the worker")
                }
            }
        }
    }

    pub fn send_message_to_worker(&mut self, worker_message: WorkerMessage) {
        self.tx
            .start_send(Message::Binary(
                bincode::serialize::<WorkerMessage>(&worker_message).unwrap(),
            ))
            .unwrap_or_else(|err| error!("{}", err));
        //TODO: Have the worker send a message to the server if it can't access the file
    }

    pub fn add_to_queue(&mut self, encode: Encode) {
        //share credentials will have to be handled on the worker side
        if !encode.source_path.exists() {
            error!(
                "source_path is not accessible from the server: {:?}",
                encode.source_path
            );
            //TODO: mark this Encode Task as failed because "file not found", change it to a
            //      state where it can be stored, then manually repaired before being restarted,
            panic!();
        }

        //Adds the encode to the workers queue server-side, this should mirror the client-side queue
        self.transcode_queue
            .write()
            .unwrap()
            .push_back(encode.clone());

        //Sends the encode to the worker
        self.send_message_to_worker(WorkerMessage::for_encode(encode));
    }

    pub fn check_if_active(&mut self) {
        //If the connection is active, do nothing.
        //TODO: If something is wrong with the connection, close the connection server-side, making this worker unavailable, start timeout for removing the work assigned to the worker.
        //    : If there's not enough work for other workers, immediately shift the encodes to other workers (put it back in the encode queue)
    }
}

pub struct WorkerManager {
    workers: VecDeque<Worker>,
    transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
}

impl WorkerManager {
    pub fn default() -> Self {
        Self {
            workers: VecDeque::new(),
            transcode_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn add_worker(&mut self, tx: UnboundedSender<Message>, transcode_queue_capacity: usize) {
        let mut new_worker = Worker::new(tx, transcode_queue_capacity);
        new_worker.send_message_to_worker(WorkerMessage::text(
            "Worker successfully initialised".to_string(),
        ));
        self.workers.push_back(new_worker);
    }

    pub fn fill_worker_transcode_queues(&mut self) {
        if self.transcode_queue.lock().unwrap().len() > 0 {
            return;
        }
        for worker in &mut self.workers {
            worker.fill_transcode_queue(self.transcode_queue.clone());
        }
    }

    pub fn send_encode_to_specific_worker(&mut self, _worker_uid: usize, _encode: Encode) {
        //TODO
    }

    //TODO: Find better name
    pub fn send_encode_to_next_available_worker(&mut self, encode: Encode) {
        for worker in &mut self.workers {
            if worker.spaces_in_queue() > 0 {
                worker.add_to_queue(encode);
                return;
            }
        }

        self.transcode_queue.lock().unwrap().push_back(encode);
    }

    pub fn send_notification_to_all_workers(&mut self) {}

    pub fn send_command_to_all_workers(&mut self) {}
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum WorkerMessageType {
    Command,
    Notification,
    AddEncode,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMessage {
    //identifier: String,
    pub text: Option<String>,
    pub encode: Option<Encode>,
    message_type: WorkerMessageType,
}

impl WorkerMessage {
    fn text(text: String) -> Self {
        Self {
            text: Some(text),
            encode: None,
            message_type: WorkerMessageType::Unknown,
        }
    }
    //do something else, like shutdown, cancel current encode, flush queue, switch to running a specific encode (regardless of progress)
    //many of these actions are destructive, many of them will technically waste CPU time
    //most functions for a worker will be handled here
    pub fn for_command(text: String) -> Self {
        WorkerMessage::text(text)
    }

    pub fn for_encode(encode: Encode) -> Self {
        Self {
            text: None,
            encode: Some(encode),
            message_type: WorkerMessageType::Unknown,
        }
    }

    //this would be used to let the worker know the server will be unavailable for x amount of time and
    //to continue to establish a websocket connection, but continue working on it's encode queue
    //or for the server to output to the workers console
    pub fn announcement(text: String) -> Self {
        WorkerMessage::text(text)
    }
}
