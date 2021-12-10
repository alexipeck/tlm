use futures_channel::mpsc::UnboundedSender;
use std::{
    collections::VecDeque,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Instant,
};
use tokio_tungstenite::tungstenite::Message;
use tracing::error;

use crate::worker_manager::AddEncodeMode;
use crate::worker_manager::Encode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Worker {
    pub uid: u32,
    pub worker_ip_address: SocketAddr,
    tx: UnboundedSender<Message>,
    pub transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
    pub close_time: Option<Instant>,
    //TODO: Time remaining on current episode
    //TODO: Current encode percentage
}

impl Worker {
    pub fn new(uid: u32, worker_ip_address: SocketAddr, tx: UnboundedSender<Message>) -> Self {
        Self {
            uid,
            worker_ip_address,
            tx,
            transcode_queue: Arc::new(RwLock::new(VecDeque::new())),
            close_time: None,
        }
    }
    //TODO: Consolidate server-side worker transcode queue and worker-side transcode queue
    pub fn clear_current_transcode(&mut self, generic_uid: usize) {
        let mut transcode_queue_lock = self.transcode_queue.write().unwrap();
        if !transcode_queue_lock.is_empty() {
            let transcode = transcode_queue_lock.remove(0).unwrap();
            if transcode.generic_uid != generic_uid {
                panic!("Server-side and worker-side transcode queues don't mirror each other.");
            }
        }
    }

    pub fn update(&mut self, worker_ip_address: SocketAddr, tx: UnboundedSender<Message>) {
        self.worker_ip_address = worker_ip_address;
        self.tx = tx;
    }

    pub fn spaces_in_queue(&mut self) -> i64 {
        2 - self.transcode_queue.read().unwrap().len() as i64
    }

    pub fn send_message_to_worker(&mut self, worker_message: VersatileMessage) {
        self.tx
            .start_send(worker_message.to_message())
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
        self.send_message_to_worker(VersatileMessage::Encode(encode, AddEncodeMode::Back));
    }

    pub fn check_if_active(&mut self) {
        //If the connection is active, do nothing.
        //TODO: If something is wrong with the connection, close the connection server-side, making this worker unavailable, start timeout for removing the work assigned to the worker.
        //    : If there's not enough work for other workers, immediately shift the encodes to other workers (put it back in the encode queue)
    }
}

///Messages to be serialised and sent between the worker and server
#[derive(Serialize, Deserialize)]
pub enum VersatileMessage {
    //Worker
    Encode(Encode, AddEncodeMode),
    Initialise(Option<u32>),
    WorkerID(u32),
    Announce(String),
    EncodeStarted(usize, usize),
    EncodeFinished(usize, usize),

    //WebUI
    EncodeGeneric(u32, AddEncodeMode),

    //Generic
    Text(String),
}

impl VersatileMessage {
    ///Convert WorkerMessage to a tungstenite message for sending over websockets
    pub fn to_message(&self) -> Message {
        let serialised = bincode::serialize(self).unwrap_or_else(|err| {
            error!("Failed to serialise WorkerMessage: {}", err);
            panic!();
        });
        Message::binary(serialised)
    }

    pub fn from_message(message: Message) -> Self {
        bincode::deserialize::<VersatileMessage>(&message.into_data()).unwrap_or_else(|err| {
            error!("Failed to deserialise message: {}", err);
            panic!();
        })
    }
}
