//!Module for handing web socket connections that will be used with
//!both the cli and web ui controller to communicate in both directions as necessary
use std::{collections::VecDeque, sync::RwLock};
use tracing::{error, info, warn, debug};

use crate::{
    config::WorkerConfig,
    database::{create_file_version, print_all_worker_models},
    file_manager::FileManager,
    generic::FileVersion,
    model::NewFileVersion,
    pathbuf_to_string,
    scheduler::{Hash, ImportFiles, ProcessNewFiles, Task, TaskType, GenerateProfiles},
    worker::WorkerMessage,
    worker_manager::{AddEncodeMode, WorkerManager, WorkerTranscodeQueue}, encode::Encode,
};

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio_tungstenite::connect_async;

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, (Option<i32>, Tx)>>>;

async fn handle_web_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    tasks: Arc<Mutex<VecDeque<Task>>>,
    file_manager: Arc<Mutex<FileManager>>,
    worker_mananger_transcode_queue: Arc<Mutex<VecDeque<Encode>>>,
    worker_manager: Arc<Mutex<WorkerManager>>,
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
                "hash" => tasks
                    .lock()
                    .unwrap()
                    .push_back(Task::new(TaskType::Hash(Hash::default()))),
                "import" => tasks
                    .lock()
                    .unwrap()
                    .push_back(Task::new(TaskType::ImportFiles(ImportFiles::default()))),
                "process" => tasks
                    .lock()
                    .unwrap()
                    .push_back(Task::new(TaskType::ProcessNewFiles(
                        ProcessNewFiles::default(),
                    ))),
                "generate_profiles" => tasks
                    .lock()
                    .unwrap()
                    .push_back(Task::new(TaskType::GenerateProfiles(
                        GenerateProfiles::default(),
                    ))),
                "display_workers" => print_all_worker_models(),
                "run_completeness_check" => {
                    fn bool_to_char(bool: bool) -> char {
                        if bool {
                            'T'
                        } else {
                            'Y'
                        }
                    }
                    fn line_output(file_version: &FileVersion) {
                        let hash = file_version.hash.is_some();
                        let fast_hash = file_version.fast_hash.is_some();
                        let width = file_version.width.is_some();
                        let height = file_version.height.is_some();
                        let framerate = file_version.framerate.is_some();
                        let length_time = file_version.length_time.is_some();
                        let resolution_standard = file_version.resolution_standard.is_some();
                        let container = file_version.container.is_some();
                        if !hash || !fast_hash || !width || !height || !framerate || !length_time || !resolution_standard || !container {
                            debug!(
                                "hash: {}, fast_hash: {}, width: {}, height: {}, framerate: {}, length_time: {}, resolution_standard: {}, container: {}",
                                bool_to_char(hash),
                                bool_to_char(fast_hash),
                                bool_to_char(width),
                                bool_to_char(height),
                                bool_to_char(framerate),
                                bool_to_char(length_time),
                                bool_to_char(resolution_standard),
                                bool_to_char(container),
                            );
                        }
                        
                    }
                    let file_manager_lock = file_manager.lock().unwrap();

                    debug!("Generics: {}", file_manager_lock.generic_files.len());
                    let mut episodes_count = 0;
                    for show in file_manager_lock.shows.iter() {
                        for season in show.seasons.iter() {
                            for episode in season.episodes.iter() {
                                episodes_count += episode.generic.file_versions.len();
                            }
                        }
                    }
                    debug!("Episodes: {}", episodes_count);

                    for generic in file_manager_lock.generic_files.iter() {
                        for file_version in generic.file_versions.iter() {
                            line_output(file_version);
                        }
                    }
                    for show in file_manager_lock.shows.iter() {
                        for season in show.seasons.iter() {
                            for episode in season.episodes.iter() {
                                for file_version in episode.generic.file_versions.iter() {
                                    line_output(file_version);
                                }
                            }
                        }
                    }
                },

                _ => warn!("{} is not a valid input", message),
            }
        } else if msg.is_binary() {
            match WorkerMessage::from_message(msg) {
                WorkerMessage::Initialise(mut worker_uid) => {
                    //if true {//TODO: authenticate/validate
                    if !worker_manager.lock().unwrap().reestablish_worker(
                        worker_uid,
                        addr,
                        tx.clone(),
                    ) {
                        //We need the new uid so we can set it correctly in the peer map
                        worker_uid =
                            Some(worker_manager.lock().unwrap().add_worker(addr, tx.clone()));
                    }
                    peer_map.lock().unwrap().get_mut(&addr).unwrap().0 = worker_uid;
                    //}
                }
                WorkerMessage::EncodeGeneric(generic_uid, file_version_id, add_encode_mode) => {
                    match file_manager
                        .lock()
                        .unwrap()
                        .get_encode_from_generic_uid(generic_uid, file_version_id)
                    {
                        Some(encode) => {
                            match add_encode_mode {
                                AddEncodeMode::Back => {
                                    worker_mananger_transcode_queue
                                        .lock()
                                        .unwrap()
                                        .push_back(encode);
                                }
                                AddEncodeMode::Next => {
                                    worker_mananger_transcode_queue
                                        .lock()
                                        .unwrap()
                                        .push_front(encode);
                                }
                                AddEncodeMode::Now => {
                                    //TODO: Implement immediate encode
                                }
                            }
                            info!("Setting up generic for transcode");
                        }
                        None => {
                            info!("No generics available to transcode");
                        }
                    }
                }
                WorkerMessage::EncodeStarted(worker_uid, generic_uid) => info!(
                    "Worker with UID: {} has started transcoding generic with UID: {}",
                    worker_uid, generic_uid
                ),
                WorkerMessage::EncodeFinished(worker_uid, generic_uid, full_path) => {
                    worker_manager
                        .lock()
                        .unwrap()
                        .clear_current_transcode_from_worker(worker_uid, generic_uid);
                    if !file_manager.lock().unwrap().insert_file_version(&FileVersion::from_file_version_model(create_file_version(NewFileVersion::new(generic_uid, pathbuf_to_string(&full_path), false)))) {
                        error!("This should've found a generic to insert it into, this shouldn't have happened.");
                        panic!();
                    }
                    //TODO: Make an enum of actions that could be performed on a Worker, like clear_current_transcode
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
) -> Result<(), IoError> {
    let addr_ipv4 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("127.0.0.1:{}", port));

    let addr_ipv6 = env::args()
        .nth(1)
        .unwrap_or_else(|| format!("[::1]:{}", port));

    let state = PeerMap::new(Mutex::new(HashMap::new()));

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
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone()));
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone()));
                }
            }
        } else if is_listening_ipv4 {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone()));
                }
            }
        } else {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_web_connection(state.clone(), stream, addr, tasks.clone(), file_manager.clone(), worker_mananger_transcode_queue.clone(), worker_manager.clone()));
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
