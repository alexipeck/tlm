use crate::generic::Generic;
use futures_channel::mpsc::UnboundedSender;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    net::SocketAddr,
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info};

pub fn generate_uid() -> String {
    //TODO: Actually unique ID not thoughts and prayers
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let uid: String = {
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    };
    uid
}

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
    uid: String,
    worker_ip_address: SocketAddr,
    tx: UnboundedSender<Message>,
    transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
    pub close_time: Option<Instant>,
    //TODO: Time remaining on current episode
    //TODO: Current encode percentage
    //TODO: Worker UID, should be based on some hardware identifier, so it can be regenerated
    //NOTE: If this is running under a Docker container, it may have a random MAC address, so on reboot,
    //    : becoming a new worker will probably mean the old one should be dropped after x amount of time.
}

impl Worker {
    pub fn new(uid: String, worker_ip_address: SocketAddr, tx: UnboundedSender<Message>) -> Self {
        Self {
            uid,
            worker_ip_address,
            tx,
            transcode_queue: Arc::new(RwLock::new(VecDeque::new())),
            close_time: None,
        }
    }

    pub fn update(&mut self, worker_ip_address: SocketAddr, tx: UnboundedSender<Message>) {
        self.worker_ip_address = worker_ip_address;
        self.tx = tx;
    }

    pub fn spaces_in_queue(&mut self) -> usize {
        //TODO: Make queue capacity come from the config file
        2 - self.transcode_queue.read().unwrap().len()
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
    worker_icu: VecDeque<Worker>,
    transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
}

impl WorkerManager {
    pub fn default() -> Self {
        Self {
            workers: VecDeque::new(),
            worker_icu: VecDeque::new(),
            transcode_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    //atm, we only care about the IP address in the SocketAddr, leaving the whole thing because deals with both IPV4 and IPV6
    pub fn add_worker(
        &mut self,
        worker_uid: String,
        worker_ip_address: SocketAddr,
        tx: UnboundedSender<Message>,
    ) {
        let mut new_worker = Worker::new(worker_uid, worker_ip_address, tx);
        new_worker.send_message_to_worker(WorkerMessage::text(
            "worker_successfully_initialised".to_string(),
        ));
        self.workers.push_back(new_worker);
    }

    pub fn reestablish_worker(
        &mut self,
        worker_uid: String,
        worker_ip_address: SocketAddr,
        tx: UnboundedSender<Message>,
    ) -> bool {
        let mut index: Option<usize> = None;
        for (i, worker) in self.worker_icu.iter_mut().enumerate() {
            if worker.uid == worker_uid {
                worker.update(worker_ip_address, tx);
                worker.close_time = None;
                index = Some(i);
                break;
            }
        }
        if index.is_none() {
            return false;
        }
        self.workers
            .push_back(self.worker_icu.remove(index.unwrap()).unwrap()); //Check if unwrapping .remove() is safe
        info!("Worker successfully re-established");
        true
    }

    pub fn drop_timed_out_workers(&mut self, timeout_threshold: u64) {
        let mut indexes: Vec<usize> = Vec::new();
        for (i, worker) in self.worker_icu.iter().enumerate() {
            //Check if worker is timed out
            if worker.close_time.is_none() {
                continue;
            }

            //Mark worker to be removed
            if worker.close_time.unwrap().elapsed().as_secs() > timeout_threshold {
                indexes.push(i);
            }
        }
        indexes.reverse();
        for index in indexes {
            if let Some(worker) = self.workers.remove(index) {
                self.transcode_queue
                    .lock()
                    .unwrap()
                    .append(&mut worker.transcode_queue.write().unwrap());
                info!("Worker ID: {} has been dropped. It's queue has been returned to the main queue", worker.uid);
            }
        }
    }

    pub fn start_worker_timeout(&mut self, worker_uid: String) {
        for (i, worker) in self.workers.iter_mut().enumerate() {
            if worker.uid == worker_uid {
                worker.close_time = Some(Instant::now());
                self.worker_icu.push_back(self.workers.remove(i).unwrap());
                return;
            }
        }
        panic!(
            "The WorkerManager couldn't find a worker associated with this worker_uid: {}",
            worker_uid
        );
    }

    pub fn round_robin_fill_transcode_queues(&mut self) {
        for worker in self.workers.iter_mut() {
            if worker.spaces_in_queue() < 1 {
                continue;
            }
            match self.transcode_queue.lock().unwrap().pop_front() {
                Some(encode) => {
                    worker.add_to_queue(encode);
                }
                None => {
                    info!("No encode tasks to send to the worker");
                    break;
                }
            }
        }
    }

    pub fn send_encode_to_specific_worker(&mut self, _worker_uid: usize, _encode: Encode) {
        //TODO
    }

    //TODO: Find better name
    pub fn send_encode_to_next_available_worker(&mut self, encode: Encode) {
        for worker in self.workers.iter_mut() {
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
    ///If the queue is at capacity, it will yield an error
    pub fn check_queue_capacity(&self) {
        if self.transcode_queue.read().unwrap().len() > 2 {
            error!("The transcode queue is at capacity, an transcode shouldn't have been sent, adding anyway.");
        }
    }

    fn clear_current_transcode(&mut self) {
        //Currently goes to the abyss
        //TODO: Store this somewhere or do something with it as a record that the worker has completed the transcode.
        let _ = self.current_transcode.write().unwrap().take();
    }

    fn start_current_transcode_if_some(&mut self) {
        if self.current_transcode.read().unwrap().is_some() {
            self.kill_current_transcode_process();
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

    pub fn run_transcode(&mut self) {
        {
            let transcode_lock = self.current_transcode.read().unwrap();
            let handle_lock = self.current_transcode_handle.read().unwrap();
            //Check the state of the current encode/handle
            if transcode_lock.is_some() && handle_lock.is_some() {
                error!("There is already an transcode running");
                return;
            }
        }

        //Add a transcode current if there isn't one already there
        //Assigns current_transcode an
        if self.make_transcode_current() {
            self.start_current_transcode_if_some();

            if self.current_transcode_handle.read().unwrap().is_some() {
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
            }
        }
    }

    ///Makes a transcode current if there isn't one already there
    ///Returns true if there is a transcode ready to go after this function has run
    fn make_transcode_current(&mut self) -> bool {
        if self.current_transcode.read().unwrap().is_some() {
            if let Some(encode) = self.transcode_queue.write().unwrap().pop_front() {
                let _ = self.current_transcode.write().unwrap().insert(encode);
            }
        }

        self.current_transcode.read().unwrap().is_some()
    }

    ///Swaps out passed in encode for the currently running one and moved it to the front of the queue
    fn replace_current_encode(&mut self, encode: Encode) {
        if let Some(current_encode) = self.current_transcode.write().unwrap().replace(encode) {
            self.transcode_queue
                .write()
                .unwrap()
                .push_front(current_encode);
        }
    }

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
    pub text: Option<String>,
    pub worker_uid: Option<String>,
    pub encode: Option<(Encode, AddEncodeMode)>,
    basic_message: bool,
}

impl WorkerMessage {
    fn text(text: String) -> Self {
        Self {
            text: Some(text),
            worker_uid: None,
            encode: None,
            basic_message: true,
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
            worker_uid: None,
            encode: Some((encode, encode_add_mode)),
            basic_message: false,
        }
    }

    pub fn for_initialisation(worker_uid: String) -> Self {
        Self {
            text: Some(String::from("initialise_worker")),
            worker_uid: Some(worker_uid),
            encode: None,
            basic_message: false,
        }
    }

    //this would be used to let the worker know the server will be unavailable for x amount of time and
    //to continue to establish a websocket connection, but continue working on it's encode queue
    //or for the server to output to the workers console
    pub fn announcement(text: String) -> Self {
        WorkerMessage::text(text)
    }
}
