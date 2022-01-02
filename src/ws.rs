//!Module for handing web socket connections that will be used with
//!both the cli and web ui controller to communicate in both directions as necessary
use std::{collections::VecDeque, sync::RwLock};
use tracing::{error, info, warn};

use crate::{
    config::{ServerConfig, WorkerConfig},
    database::print_all_worker_models,
    debug::{
        encode_all_files, output_all_file_versions, output_tracked_paths, run_completeness_check,
    },
    encode::Encode,
    file_manager::FileManager,
    scheduler::Task,
    unit_tests::file_access_self_test,
    worker::WorkerMessage,
    worker_manager::{WorkerManager, WorkerTranscodeQueue},
    ws_functions::{
        encode_finished, encode_generic, encode_started, generate_profiles, hash_files,
        import_files, initialise, move_finished, move_started, process_files,
    },
    PeerMap,
};

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::connect_async;

use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio_tungstenite::tungstenite::protocol::Message;

#[allow(clippy::too_many_arguments)]
async fn handle_web_connection(
    peer_map: Arc<Mutex<PeerMap>>,
    raw_stream: TcpStream,
    addr: SocketAddr,
    tasks: Arc<Mutex<VecDeque<Task>>>,
    file_manager: Arc<Mutex<FileManager>>,
    worker_manager_transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    worker_manager: Arc<Mutex<WorkerManager>>,
    server_config: Arc<RwLock<ServerConfig>>,
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
    peer_map.lock().unwrap().insert(addr, (None, tx.clone()));
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if msg.is_text() {
            let message = msg
                .to_text()
                .unwrap()
                .strip_suffix("\r\n")
                .or_else(|| msg.to_text().unwrap().strip_suffix('\n'))
                .unwrap_or_else(|| msg.to_text().unwrap());
            match message {
                //Tasks
                "hash" => hash_files(tasks.clone()),
                "import" => import_files(tasks.clone()),
                "process" => process_files(tasks.clone()),
                "generate_profiles" => generate_profiles(tasks.clone()),
                "bulk" => {
                    //TODO: Implement a way of making one task wait for another before it can run
                    //    : this will require tasks to be logged in the DB and knowledge of the uid for the await
                    //    : this is a scheduler/task thing
                    import_files(tasks.clone());
                    process_files(tasks.clone());
                    hash_files(tasks.clone());
                    generate_profiles(tasks.clone());
                }

                //Debug tasks
                "output_tracked_paths" => output_tracked_paths(file_manager.clone()),
                "output_file_versions" => output_all_file_versions(file_manager.clone()),
                "display_workers" => print_all_worker_models(),
                "encode_all" => {
                    encode_all_files(file_manager.clone(), worker_manager_transcode_queue.clone())
                }
                "run_completeness_check" => run_completeness_check(file_manager.clone()),
                "kill_all_workers" => {
                    //TODO: Make this force close all workers, used for constant resetting of the dev/test environment
                }

                //Self test
                "file_access_self_test" => {
                    let _ = file_access_self_test(server_config.clone());
                }

                _ => warn!("{} is not a valid input", message),
            }
        } else if msg.is_binary() {
            let worker_message = WorkerMessage::from_message(msg);
            match worker_message {
                WorkerMessage::Initialise(_, _) => {
                    initialise(
                        worker_message,
                        worker_manager.clone(),
                        addr,
                        tx.clone(),
                        peer_map.clone(),
                    );
                }
                WorkerMessage::EncodeGeneric(_, _, _, _) => {
                    encode_generic(
                        worker_message,
                        file_manager.clone(),
                        worker_manager_transcode_queue.clone(),
                        server_config.clone(),
                    );
                }
                WorkerMessage::EncodeStarted(_, _) => {
                    encode_started(worker_message);
                }
                WorkerMessage::EncodeFinished(_, _, _) => {
                    encode_finished(worker_message);
                }
                WorkerMessage::MoveStarted(_, _, _, _) => {
                    move_started(worker_message);
                }
                WorkerMessage::MoveFinished(_, _, _) => {
                    move_finished(worker_message, worker_manager.clone(), file_manager.clone());
                }
                _ => {
                    warn!("Server recieved a message it doesn't know how to handle");
                }
            }
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    let mut lock = peer_map.lock().unwrap();
    //The worker should always exist for as long as the connection exists
    if let Some(to_remove) = lock.get(&addr).unwrap().0 {
        //TODO: Only have it remove the worker if it hasn't reestablished the connection withing x amount of time
        worker_manager
            .lock()
            .unwrap()
            .start_worker_timeout(to_remove);
    }

    lock.remove(&addr);
}

pub async fn run_web(
    port: u16,
    tasks: Arc<Mutex<VecDeque<Task>>>,
    file_manager: Arc<Mutex<FileManager>>,
    worker_mananger_transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    worker_manager: Arc<Mutex<WorkerManager>>,
    server_config: Arc<RwLock<ServerConfig>>,
) -> Result<(), IoError> {
    let addr_ipv4 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("127.0.0.1:{}", port));

    let addr_ipv6 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("[::1]:{}", port));

    let state = Arc::new(Mutex::new(HashMap::new()));

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
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone(), server_config.clone()));
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone(), server_config.clone()));
                }
            }
        } else if is_listening_ipv4 {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone(), server_config.clone()));
                }
            }
        } else {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone(), server_config.clone()));
                }
            }
        }
    }

    //Close all websocket connection gracefully before exit
    for (_, tx) in (&mut *state.lock().unwrap()).values_mut() {
        let _ = tx.start_send(Message::Close(None));
    }

    Ok(())
}

pub async fn run_worker(
    transcode_queue: Arc<RwLock<WorkerTranscodeQueue>>,
    rx: futures_channel::mpsc::UnboundedReceiver<Message>,
    config: Arc<RwLock<WorkerConfig>>,
) -> Result<(), IoError> {
    let url = url::Url::parse(&config.read().unwrap().to_string()).unwrap();

    let ws_stream;
    match connect_async(url).await {
        Ok((stream, _)) => ws_stream = stream,
        Err(_) => return Ok(()),
    }
    info!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();
    let stdin_to_ws = rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            //TODO: Handle inputs (likely shared memory or another mpsc)
            let message = message.unwrap();
            if message.is_close() {
                info!("Server has disconnected voluntarily");
                //TODO: Trigger the worker to start trying to reestablish the connection
                //      It should continue a running transcode, but ONLY complete the current transcode until the server connection has been established
                info!("Worker is beginning to try and reestablish a connection to the server");
                info!("Worker is continuing it's current transcode");
                return;
            }
            match WorkerMessage::from_message(message) {
                WorkerMessage::Encode(encode, add_encode_mode) => {
                    transcode_queue
                        .write()
                        .unwrap()
                        .add_encode(encode, add_encode_mode);
                }
                WorkerMessage::WorkerID(worker_uid) => {
                    config.write().unwrap().uid = Some(worker_uid);
                    config.read().unwrap().update_config_on_disk();
                    info!("Worker has been given UID: {}", worker_uid);
                }
                WorkerMessage::Announce(text) => {
                    info!("Announcement: {}", text);
                }
                _ => warn!("Worker received a message it doesn't know how to handle"),
            }
        })
    };
    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;

    Ok(())
}
