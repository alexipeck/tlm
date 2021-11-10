//!Module for handing web socket connections that will be used with
//!both the cli and web ui controller to communicate in both directions as necessary
use std::collections::VecDeque;
use tracing::{error, info, warn};

use crate::scheduler::{Encode, Hash, ImportFiles, ProcessNewFiles, Task, TaskType};

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<SocketAddr, Tx>>>;

async fn handle_web_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    tasks: Arc<RwLock<VecDeque<Task>>>,
) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .unwrap_or_else(|err| {
            error!(
                "Error during the websocket handshake occurred. Err: {}",
                err
            );
            panic!();
        });
    info!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.write().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let message = msg
            .to_text()
            .unwrap()
            .strip_suffix("\r\n")
            .or_else(|| msg.to_text().unwrap().strip_suffix('\n'))
            .unwrap_or_else(|| msg.to_text().unwrap());

        info!("Received a message from {}: {}", addr, message);

        match message {
            "hash" => {
                tasks
                    .write()
                    .unwrap()
                    .push_back(Task::new(TaskType::Hash(Hash::default())));
            }
            "import" => tasks
                .write()
                .unwrap()
                .push_back(Task::new(TaskType::ImportFiles(ImportFiles::default()))),
            "process" => tasks
                .write()
                .unwrap()
                .push_back(Task::new(TaskType::ProcessNewFiles(
                    ProcessNewFiles::default(),
                ))),
            //TODO: Encode message needs a UID for transcoding a specific generic/episode
            //"encode" => encode_tasks.lock().unwrap().push_back(Task::new(TaskType::Encode(Encode::new())))
            _ => warn!("{} is not a valid input", message),
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    peer_map.write().unwrap().remove(&addr);
}

pub async fn run_web(port: u16, tasks: Arc<RwLock<VecDeque<Task>>>) -> Result<(), IoError> {
    let addr_ipv4 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("127.0.0.1:{}", port));

    let addr_ipv6 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("[::1]:{}", port));

    let state = PeerMap::new(RwLock::new(HashMap::new()));

    // Create the event loop and TCP listener
    let try_socket_ipv4 = TcpListener::bind(&addr_ipv4).await;
    let listener_ipv4 = try_socket_ipv4;
    let try_socket_ipv6 = TcpListener::bind(&addr_ipv6).await;
    let listener_ipv6 = try_socket_ipv6;

    let mut is_listening_ipv4 = false;
    let mut is_listening_ipv6 = false;

    if listener_ipv4.is_ok() {
        is_listening_ipv4 = true;
        info!("Listening on: {}", addr_ipv4);
    } else {
        warn!("Failed to bind to ipv4: {}", addr_ipv4);
    }

    if listener_ipv6.is_ok() {
        is_listening_ipv6 = true;
        info!("Listening on: {}", addr_ipv6);
    } else {
        warn!("Failed to bind to ipv6: {}", addr_ipv6);
    }

    if !is_listening_ipv4 && !is_listening_ipv6 {
        error!(
            "Could not bind to {} or {}. Websocket connections not possible",
            addr_ipv6, addr_ipv4
        );
    }

    //Handle ipv4 and ipv6 simultaneously and end if ctrl_c is run
    //
    //This looks and is a bit janky. Need to look into a way of specifying
    //a set of tasks for a select fo listen to based on a condition instead
    //of using 3 select macros. For now this will work
    loop {
        if is_listening_ipv4 && is_listening_ipv6 {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C recieved, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone()));
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone()));
                }
            }
        } else if is_listening_ipv4 {
            tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Ctrl-C recieved, shutting down");
                break;
            }
            Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone()));
            }
            }
        } else {
            tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Ctrl-C recieved, shutting down");
                break;
            }
            Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone()));
            }
            }
        }
    }

    Ok(())
}

async fn handle_worker_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    tasks: Arc<RwLock<VecDeque<Encode>>>,
) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .unwrap_or_else(|err| {
            error!(
                "Error during the websocket handshake occurred. Err: {}",
                err
            );
            panic!();
        });
    info!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.write().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let message = msg.into_data();
        info!("Received a message from {}", addr);

        match bincode::deserialize::<Encode>(&message) {
            Ok(encode) => {
                tasks.write().unwrap().push_back(encode);
            }
            Err(err) => {
                warn!("Error when deserializing encode from websocket: {}", err);
            }
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    peer_map.write().unwrap().remove(&addr);
}

pub async fn run_worker(
    port: u16,
    transcode_queue: Arc<RwLock<VecDeque<Encode>>>,
) -> Result<(), IoError> {
    let addr_ipv4 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("127.0.0.1:{}", port));

    let addr_ipv6 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("[::1]:{}", port));

    let state = PeerMap::new(RwLock::new(HashMap::new()));

    // Create the event loop and TCP listener
    let try_socket_ipv4 = TcpListener::bind(&addr_ipv4).await;
    let listener_ipv4 = try_socket_ipv4;
    let try_socket_ipv6 = TcpListener::bind(&addr_ipv6).await;
    let listener_ipv6 = try_socket_ipv6;

    let mut is_listening_ipv4 = false;
    let mut is_listening_ipv6 = false;

    if listener_ipv4.is_ok() {
        is_listening_ipv4 = true;
        info!("Listening on: {}", addr_ipv4);
    } else {
        warn!("Failed to bind to ipv4: {}", addr_ipv4);
    }

    if listener_ipv6.is_ok() {
        is_listening_ipv6 = true;
        info!("Listening on: {}", addr_ipv6);
    } else {
        warn!("Failed to bind to ipv6: {}", addr_ipv6);
    }

    if !is_listening_ipv4 && !is_listening_ipv6 {
        error!(
            "Could not bind to {} or {}. Websocket connections not possible",
            addr_ipv6, addr_ipv4
        );
    }

    //Handle ipv4 and ipv6 simultaneously and end if ctrl_c is run
    //
    //This looks and is a bit janky. Need to look into a way of specifying
    //a set of tasks for a select fo listen to based on a condition instead
    //of using 3 select macros. For now this will work
    loop {
        if is_listening_ipv4 && is_listening_ipv6 {
            tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Ctrl-C recieved, shutting down");
                break;
            }
            Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                tokio::spawn(handle_worker_connection(state.clone(), stream, addr, transcode_queue.clone()));
            }
            Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                tokio::spawn(handle_worker_connection(state.clone(), stream, addr, transcode_queue.clone()));
            }
            }
        } else if is_listening_ipv4 {
            tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Ctrl-C recieved, shutting down");
                break;
            }
            Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                tokio::spawn(handle_worker_connection(state.clone(), stream, addr, transcode_queue.clone()));
            }
            }
        } else {
            tokio::select! {
            _ = signal::ctrl_c() => {
                warn!("Ctrl-C recieved, shutting down");
                break;
            }
            Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                tokio::spawn(handle_worker_connection(state.clone(), stream, addr, transcode_queue.clone()));
            }
            }
        }
    }

    Ok(())
}
