use crate::database::get_all_workers;
use crate::database::{create_worker, establish_connection};
use crate::generic::Generic;
use crate::model::NewWorker;
use crate::worker::{VersatileMessage, Worker};
use futures_channel::mpsc::UnboundedSender;
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
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encode {
    pub generic_uid: i32,
    pub source_path: PathBuf,
    pub future_filename: String,
    pub encode_options: Vec<String>,
    //pub profile: Profile,
}

impl Encode {
    pub fn new(generic_uid: i32, source_path: PathBuf, future_filename: String, encode_options: Vec<String>) -> Self {
        Self {
            generic_uid,
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

pub enum WorkerAction {
    ClearCurrentTranscode(i32),
}

pub struct WorkerManager {
    workers: Arc<Mutex<VecDeque<Worker>>>,
    closed_workers: VecDeque<Worker>,
    transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    timeout_threshold: u64,
}

impl WorkerManager {
    pub fn new(
        workers: Arc<Mutex<VecDeque<Worker>>>,
        transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
        timeout_threshold: u64,
    ) -> Self {
        Self {
            workers,
            closed_workers: get_all_workers(),
            transcode_queue,
            timeout_threshold,
        }
    }

    pub fn perform_on_worker(&mut self, worker_uid: Option<i32>, mut worker_actions: Vec<WorkerAction>) {
        if worker_uid.is_none() {
            panic!("The server was asked to run actions on an empty worker_uid. | None Option<i32>");
        }
        for worker in self.workers.lock().unwrap().iter_mut() {
            if worker.uid == worker_uid {
                while !worker_actions.is_empty() {
                    if let Some(worker_action) = worker_actions.pop() {
                        match worker_action {
                            WorkerAction::ClearCurrentTranscode(generic_uid) => {
                                worker.clear_current_transcode(generic_uid);
                            }
                        }
                    }
                }
                return;
            }
        }
        panic!("Worker with UID: {} was not found", worker_uid.unwrap());
    }

    pub fn clear_current_transcode_from_worker(&mut self, worker_uid: i32, generic_uid: i32) {
        let worker_uid: Option<i32> = Some(worker_uid);
        let mut worker_lock = self.workers.lock().unwrap();
        for worker in worker_lock.iter_mut() {
            if worker.uid == worker_uid {
                worker.clear_current_transcode(generic_uid);
            }
        }
    }

    //atm, we only care about the IP address in the SocketAddr, leaving the whole thing because it deals with both IPV4 and IPV6
    pub fn add_worker(
        &mut self,
        worker_ip_address: SocketAddr,
        tx: UnboundedSender<Message>,
    ) -> i32 {
        let connection = establish_connection();
        let mut new_worker = Worker::new(None, worker_ip_address, tx);
        let new_id = create_worker(&connection, NewWorker::from_worker(new_worker.clone()));
        new_worker.uid = Some(new_id);
        new_worker.send_message_to_worker(VersatileMessage::WorkerID(new_worker.uid.unwrap()));
        new_worker.send_message_to_worker(VersatileMessage::Announce(
            "Worker successfully initialised".to_string(),
        ));
        self.workers.lock().unwrap().push_back(new_worker);
        new_id
    }

    pub fn polling_event(&mut self) {
        self.drop_timed_out_workers();
        self.fill_transcode_queues();
    }

    pub fn reestablish_worker(
        &mut self,
        worker_uid: Option<i32>,
        worker_ip_address: SocketAddr,
        tx: UnboundedSender<Message>,
    ) -> bool {
        //Worker can't be reestablished if it doesn't have/send a uid
        if worker_uid.is_none() {
            return false;
        };
        let mut index: Option<usize> = None;
        for (i, worker) in self.closed_workers.iter_mut().enumerate() {
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

        let mut reestablished_worker = self.closed_workers.remove(index.unwrap()).unwrap();
        reestablished_worker.send_message_to_worker(VersatileMessage::Announce(
            "Worker successfully re-established".to_string(),
        ));
        self.workers.lock().unwrap().push_back(reestablished_worker); //Check if unwrapping .remove() is safe
        info!("Worker successfully re-established");
        true
    }

    pub fn drop_timed_out_workers(&mut self) {
        let mut indexes: Vec<usize> = Vec::new();
        for (i, worker) in self.closed_workers.iter_mut().enumerate() {
            //Check if worker has been timed out
            if worker.close_time.is_none() {
                continue;
            }

            //Clear worker queue and add it back to main queue
            if worker.close_time.unwrap().elapsed().as_secs() > self.timeout_threshold {
                indexes.push(i);
            } 
        }
        indexes.reverse();
        for index in indexes {
            if let Some(mut worker) = self.closed_workers.remove(index) {
                self.transcode_queue
                    .lock()
                    .unwrap()
                    .append(&mut worker.transcode_queue.write().unwrap());
                //clear the workers queue
                worker.transcode_queue.write().unwrap().clear();
                worker.close_time = None;
            }
        }
    }

    pub fn start_worker_timeout(&mut self, worker_uid: i32) {
        let mut workers_lock = self.workers.lock().unwrap();
        for (index, worker) in workers_lock.iter_mut().enumerate() {
            if worker.uid.unwrap() == worker_uid {
                worker.close_time = Some(Instant::now());
                self.closed_workers
                    .push_back(workers_lock.remove(index).unwrap());
                return;
            }
        }
        warn!(
            "The WorkerManager couldn't find a worker associated with this worker_uid: {}",
            worker_uid
        );
    }

    ///Uses Round-robin fill method
    pub fn fill_transcode_queues(&mut self) {
        for worker in self.workers.lock().unwrap().iter_mut() {
            if worker.spaces_in_queue() < 1 {
                continue;
            }
            match self.transcode_queue.lock().unwrap().pop_front() {
                Some(encode) => {
                    worker.add_to_queue(encode);
                }
                None => {
                    break;
                }
            }
        }
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

    pub fn clear_current_transcode(&mut self) {
        //Currently goes to the abyss
        //TODO: Store this somewhere or do something with it as a record that the worker has completed the transcode.
        let _ = self.current_transcode.write().unwrap().take();
    }

    fn start_current_transcode_if_some(&mut self) {
        if self.current_transcode.read().unwrap().is_some() {
            if self.current_transcode_handle.read().unwrap().is_some() {
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

    pub fn run_transcode(&mut self, worker_uid: Arc<RwLock<Option<i32>>>, mut tx: UnboundedSender<Message>) {
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
            let _ = tx.start_send(VersatileMessage::EncodeStarted(worker_uid.read().unwrap().unwrap(), self.current_transcode.read().unwrap().as_ref().unwrap().generic_uid).to_message());
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
                    //TODO: Send message to server that encode has finished.
                    let _ = tx.start_send(VersatileMessage::EncodeFinished(worker_uid.read().unwrap().unwrap(), self.current_transcode.read().unwrap().as_ref().unwrap().generic_uid).to_message());
                }
            }
        }
    }

    ///Makes a transcode current if there isn't one already there
    ///Returns true if there is a transcode ready to go after this function has run
    fn make_transcode_current(&mut self) -> bool {
        if self.current_transcode.read().unwrap().is_none() {
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

    pub fn add_encode(&mut self, encode: Encode, add_encode_mode: AddEncodeMode) {
        match add_encode_mode {
            AddEncodeMode::Back => {
                self.check_queue_capacity();
                self.transcode_queue.write().unwrap().push_back(encode);
            }
            AddEncodeMode::Next => {
                self.check_queue_capacity();
                self.transcode_queue.write().unwrap().push_front(encode);
            }

            AddEncodeMode::Now => {
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
    Now,
}
