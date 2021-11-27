use futures_channel::mpsc::UnboundedSender;
use std::{collections::VecDeque, net::SocketAddr, sync::{Arc, Mutex, RwLock}, time::Instant};
use tokio_tungstenite::tungstenite::Message;
use tracing::error;

use crate::{config::WorkerConfig, worker_manager::AddEncodeMode};
use crate::worker_manager::Encode;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Worker {
    pub uid: usize,
    worker_ip_address: SocketAddr,
    tx: UnboundedSender<Message>,
    pub transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
    pub close_time: Option<Instant>,
    //TODO: Time remaining on current episode
    //TODO: Current encode percentage
    //TODO: Worker UID, should be based on some hardware identifier, so it can be regenerated
    //NOTE: If this is running under a Docker container, it may have a random MAC address, so on reboot,
    //    : becoming a new worker will probably mean the old one should be dropped after x amount of time.
}

impl Worker {
    pub fn new(uid: usize, worker_ip_address: SocketAddr, tx: UnboundedSender<Message>) -> Self {
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

    pub fn spaces_in_queue(&mut self) -> i64 {
        2 - self.transcode_queue.read().unwrap().len() as i64
    }

    pub fn send_message_to_worker(&mut self, worker_message: WorkerMessage) {
        self.tx
            .start_send(worker_message.as_message())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMessage {
    pub text: Option<String>,
    pub worker_uid: Option<usize>,
    pub encode: Option<(Encode, AddEncodeMode)>,
    basic_message: bool,
}

impl WorkerMessage {
    pub fn as_message(&self) -> Message {
        let serialised = bincode::serialize(self).unwrap_or_else(|err| {
            error!("Failed to serialize WorkerMessage: {}", err);
            panic!();
        });
        Message::binary(serialised)
    }
    pub fn text(text: String) -> Self {
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

    pub fn for_initialisation(config: Arc<RwLock<WorkerConfig>>) -> Self {
        Self {
            text: Some(String::from("initialise_worker")),
            worker_uid: config.read().unwrap().uid,
            encode: None,
            basic_message: false,
        }
    }

    pub fn for_id_return(uid: usize) -> Self {
        Self {
            text: None,
            worker_uid: Some(uid),
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
