use crate::generic::Generic;
use futures_channel::mpsc::UnboundedSender;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    path::PathBuf,
    process::{Child, Command},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info};

static WORKER_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

///Struct to represent a file encode task. This is needed so we can have an enum
///that contains all types of task
///This should probably handle it's current variables without having them passed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub source_path: PathBuf,
    pub future_filename: String,
    pub encode_options: Vec<String>,
    //pub profile: Profile,
}

impl Encode {
    pub fn new(source_path: PathBuf, future_filename: String, encode_options: Vec<String>) -> Self {
        Self {
            source_path,
            future_filename,
            encode_options,
        }
    }

    ///Write lock
    pub fn run(self, handle: Arc<RwLock<Option<Child>>>) {
        info!(
            "Encoding file \'{}\'",
            Generic::get_filename_from_pathbuf(self.source_path.clone())
        );

        let _ = handle.write().unwrap().insert(
            Command::new("ffmpeg")
                .args(&self.encode_options)
                .spawn()
                .unwrap(),
        );
    }
}

#[derive(Debug)] //, Serialize, Deserialize
pub struct Worker {
    uid: usize,
    tx: UnboundedSender<Message>,
    transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
    //TODO: Time remaining on current episode
    //TODO: Current encode percentage
    //TODO: Worker UID, should be based on some hardware identifier, so it can be regenerated
    //NOTE: If this is running under a Docker container, it may have a random MAC address, so on reboot,
    //    : becoming a new worker will probably mean the old one should be dropped after x amount of time.
}

impl Worker {
    pub fn new(tx: UnboundedSender<Message>) -> Self {
        Self {
            uid: WORKER_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            tx,
            transcode_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    ///Write lock
    pub fn spaces_in_queue(&mut self) -> usize {
        //TODO: Make queue capacity come from the config file
        2 - self.transcode_queue.read().unwrap().len()
    }

    ///Write lock
    ///Returns true if there was no encodes in the queue
    pub fn add_transcode_to_queue(
        &mut self,
        transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    ) -> bool {
        match transcode_queue.lock().unwrap().pop_front() {
            Some(encode) => {
                self.transcode_queue.write().unwrap().push_back(encode);
            }
            None => {
                info!("No encode tasks to send to the worker");
                return true;
            }
        }
        false
    }

    ///Write lock
    ///Returns true if there was no encodes in the queue
    pub fn fill_transcode_queue(&mut self, transcode_queue: Arc<Mutex<VecDeque<Encode>>>) -> bool {
        for _ in 0..self.spaces_in_queue() {
            match transcode_queue.lock().unwrap().pop_front() {
                Some(encode) => {
                    self.transcode_queue.write().unwrap().push_back(encode);
                }
                None => {
                    info!("No encode tasks to send to the worker");
                    return true;
                }
            }
        }
        false
    }

    pub fn send_message_to_worker(&mut self, worker_message: WorkerMessage) {
        self.tx
            .start_send(Message::Binary(
                bincode::serialize::<WorkerMessage>(&worker_message).unwrap(),
            ))
            .unwrap_or_else(|err| error!("{}", err));
        //TODO: Have the worker send a message to the server if it can't access the file
    }

    ///Write lock
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
        self.send_message_to_worker(WorkerMessage::for_encode(encode, AddEncodeMode::Back));
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

    pub fn add_worker(&mut self, tx: UnboundedSender<Message>) -> usize {
        let mut new_worker = Worker::new(tx);
        let uid = new_worker.uid;
        new_worker.send_message_to_worker(WorkerMessage::text(
            "Worker successfully initialised".to_string(),
        ));
        self.workers.push_back(new_worker);
        uid
    }

    pub fn remove_worker(&mut self, uid: usize) {
        for (i, worker) in self.workers.iter().enumerate() {
            if worker.uid == uid {
                self.workers.remove(i);
                break;
            }
        }
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

pub struct WorkerTranscodeQueue {
    pub current_transcode: RwLock<Option<Encode>>,
    pub current_transcode_handle: Arc<RwLock<Option<Child>>>,
    pub transcode_queue: RwLock<VecDeque<Encode>>,
}

impl WorkerTranscodeQueue {
    pub fn default() -> Self {
        Self {
            current_transcode: RwLock::new(None),
            current_transcode_handle: Arc::new(RwLock::new(None)),
            transcode_queue: RwLock::new(VecDeque::new()),
        }
    }

    //Current transcode handle control
    ///Write lock
    ///Kills the currently running encode and removes the handle
    fn kill_current_transcode_process(&mut self) {
        let handle = self.current_transcode_handle.write().unwrap().take();
        if let Some(mut handle) = handle {
            match handle.kill() {
                Ok(_) => {
                    info!("Killed the currently running transcode.");
                }
                Err(err) => {
                    error!("{}", err);
                }
            }
        }
    }

    ///Read-only lock
    fn handle_is_some(&self) -> bool {
        self.current_transcode_handle.read().unwrap().is_some()
    }

    ///Read-only lock
    fn handle_is_none(&self) -> bool {
        self.current_transcode_handle.read().unwrap().is_none()
    }

    //Current transcode control
    ///Read-only lock
    fn current_transcode_is_some(&self) -> bool {
        self.current_transcode.read().unwrap().is_some()
    }

    ///Read-only lock
    fn current_transcode_is_none(&self) -> bool {
        self.current_transcode.read().unwrap().is_none()
    }

    ///Read-only lock
    ///If the queue is at capacity, it will yield an error
    pub fn check_queue_capacity(&self) {
        if self.transcode_queue.read().unwrap().len() > 2 {
            error!("The transcode queue is at capacity, an transcode shouldn't have been sent, adding anyway.");
        }
    }

    ///Write lock
    fn clear_current_transcode(&mut self) {
        //Currently goes to the abyss
        //TODO: Store this somewhere or do something with it as a record that the worker has completed the transcode.
        let _ = self.current_transcode.write().unwrap().take();
    }

    ///Write lock
    fn start_current_transcode_if_some(&mut self) {
        if self.current_transcode_is_some() {
            if self.handle_is_some() {
                self.kill_current_transcode_process();
            }
            self.current_transcode
                .write()
                .unwrap()
                .clone()
                .unwrap()
                .run(self.current_transcode_handle.clone());
        } else {
            debug!("There is no transcode available to start.");
        }
    }

    ///Write lock
    pub fn run_transcode(&mut self) {
        //Check the state of the current encode/handle
        if self.current_transcode_is_some() && self.handle_is_some() {
            error!("There is already an transcode running");
            return;
        }

        //Add a transcode current if there isn't one already there
        //Assigns current_transcode an
        if self.make_transcode_current() {
            self.start_current_transcode_if_some();

            if self.handle_is_some() {
                let output = self
                    .current_transcode_handle
                    .clone()
                    .write()
                    .unwrap()
                    .take()
                    .unwrap()
                    .wait_with_output();
                let ok: bool = output.is_ok();
                let _ = output.unwrap_or_else(|err| {
                    error!("Failed to execute ffmpeg process. Err: {}", err);
                    panic!();
                });

                if ok {
                    self.clear_current_transcode();
                }

                //only uncomment if you want disgusting output
                //should be error, but from ffmpeg, stderr mostly consists of stdout information
            }
        }
    }

    ///Write lock
    ///Makes a transcode current if there isn't one already there
    ///Returns true if there is a transcode ready to go after this function has run
    fn make_transcode_current(&mut self) -> bool {
        if self.current_transcode_is_none() {
            if let Some(encode) = self.transcode_queue.write().unwrap().pop_front() {
                let _ = self.current_transcode.write().unwrap().insert(encode);
            }
        }

        self.current_transcode_is_some()
    }

    ///Write lock
    ///Swaps out passed in encode for the currently running one and moved it to the front of the queue
    fn replace_current_encode(&mut self, encode: Encode) {
        if let Some(current_encode) = self.current_transcode.write().unwrap().replace(encode) {
            self.transcode_queue
                .write()
                .unwrap()
                .push_front(current_encode);
        }
    }

    ///Write lock
    pub fn add_encode(&mut self, encode: (Encode, AddEncodeMode)) {
        let (encode, add_encode_mode) = encode;
        match add_encode_mode {
            AddEncodeMode::Back => {
                self.check_queue_capacity();
                self.transcode_queue.write().unwrap().push_back(encode);
            }
            AddEncodeMode::Next => {
                self.check_queue_capacity();
                self.transcode_queue.write().unwrap().push_front(encode);
            }

            AddEncodeMode::NowBasic => {
                //Kill currently running encode and remove the handle
                self.kill_current_transcode_process();

                //Push the currently running encode back to the front of the queue if there is one running
                self.replace_current_encode(encode);

                //TODO: Trigger encode start
                //TODO: Store new handle
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddEncodeMode {
    Back,
    Next,
    NowBasic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMessage {
    //identifier: String,
    pub text: Option<String>,
    pub encode: Option<(Encode, AddEncodeMode)>,
}

impl WorkerMessage {
    fn text(text: String) -> Self {
        Self {
            text: Some(text),
            encode: None,
        }
    }
    //do something else, like shutdown, cancel current encode, flush queue, switch to running a specific encode (regardless of progress)
    //many of these actions are destructive, many of them will technically waste CPU time
    //most functions for a worker will be handled here
    pub fn for_command(text: String) -> Self {
        WorkerMessage::text(text)
    }

    pub fn for_encode(encode: Encode, encode_add_mode: AddEncodeMode) -> Self {
        Self {
            text: None,
            encode: Some((encode, encode_add_mode)),
        }
    }

    //this would be used to let the worker know the server will be unavailable for x amount of time and
    //to continue to establish a websocket connection, but continue working on it's encode queue
    //or for the server to output to the workers console
    pub fn announcement(text: String) -> Self {
        WorkerMessage::text(text)
    }
}
