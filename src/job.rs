use crate::{
    task::Task,
    content::Content,
    print::{print, From, Verbosity},
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering},
    process::Command,
    path::PathBuf,
};

static JOB_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct Job {
    pub uid: usize,
    pub tasks: VecDeque<Task>,

    pub source_path: PathBuf,
    pub encode_path: PathBuf,
    pub encode_string: Vec<String>,
    pub cache_directory: Option<String>,

    pub worker: Option<(usize, String)>,
    pub status_underway: bool,
    pub status_completed: bool,
}

impl Job {
    //maybe best to use a generic string
    pub fn new(source_path: PathBuf, encode_string: Vec<String>) -> Job {
        //default
        let tasks: VecDeque<Task> = VecDeque::new();

        Job {
            uid: JOB_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            tasks: VecDeque::new(),
            source_path: source_path.clone(),
            encode_path: Content::generate_encode_path_from_pathbuf(source_path),
            encode_string: encode_string,
            cache_directory: None,
            worker: None,
            status_underway: false,
            status_completed: false,
        }
    }

    /* pub fn conver_encode_string_to_vec(&mut self) -> String {

    } */

    pub fn convert_encode_string_to_actual_string(input: Vec<String>) -> String {
        let mut temp: String = String::new();
        for component in &input {
            temp += " ";
            temp += component;
        }
        return temp;
    }

    pub fn prepare_tasks(
        &mut self,
        (worker_uid, worker_string_id): (usize, String),
        cache_directory: Option<String>,
    ) {
        //eventually move first (to cache)
        self.worker = Some((worker_uid, worker_string_id));
        if cache_directory.is_some() {
            self.cache_directory = Some(cache_directory.unwrap());
        }
        self.tasks.push_back(Task::Reserve);
        self.tasks.push_back(Task::Encode);
        self.tasks.push_back(Task::Delete);
        self.tasks.push_back(Task::Move);
    }

    pub fn print(&self, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("print");
        print(
            Verbosity::INFO,
            From::Content,
            called_from,
            Content::get_filename_from_pathbuf(self.source_path.clone()),
        );
    }

    pub fn encode(&self, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("encode");
        print(
            Verbosity::INFO,
            From::Job,
            called_from,
            format!(
                "Encoding file \'{}\'",
                Content::get_filename_from_pathbuf(self.source_path.clone())
            ),
        );

        let buffer;
        if !cfg!(target_os = "windows") {
            //linux & friends
            buffer = Command::new("ffmpeg")
                .args(&self.encode_string)
                .output()
                .expect("failed to execute process");
        } else {
            //windows
            buffer = Command::new("ffmpeg")
                .args(&self.encode_string)
                .output()
                .expect("failed to execute process");
        }
        //print(Verbosity::INFO, "content", "encode", format!("{}", String::from_utf8_lossy(&buffer.stderr).to_string())); //should be error, but from ffmpeg, stderr mostly consists of stdout information
    }

    pub fn reserve(&mut self, worker: (usize, String)) {
        self.worker = Some(worker);
    }

    /* pub fn reserve(&mut self, operator: String) {
        self.reserved_by = Some(operator);
        self.underway_status = true;//bye bye
        print(Verbosity::INFO, "content", "handle", format!("reserved job UID#: {} for {}", self.uid, operator.clone()));
    } */

    pub fn handle(&mut self, worker: (usize, String), called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("handle");
        print(
            Verbosity::INFO,
            From::Job,
            called_from.clone(),
            format!("starting encoding job UID#: {} by {}", self.uid, worker.1),
        );
        self.encode(called_from.clone());
        print(
            Verbosity::INFO,
            From::Job,
            called_from.clone(),
            format!("completed encoding job UID#: {}", self.uid),
        );

        let source_path = self.source_path.to_string_lossy().to_string();
        let encode_path = self.encode_path.to_string_lossy().to_string();

        //remove source
        //move/copy encoded file to original filename with new extension (extension is currently the problem)
        //remove encoded file if it still exists

        //TODO: need to find the content entry in the db and update the path to include the new filename, most importantly the extension

        let copy_error = std::fs::copy(&encode_path, &source_path);
        match copy_error {
            Ok(file) => file,
            Err(error) => {
                print(
                    Verbosity::ERROR,
                    From::Content,
                    called_from,
                    format!("Source: {}\nDestination: {}", &source_path, &encode_path),
                );
                panic!("Problem copying the file: {:?}", error);
            }
        };
        let remove_error = std::fs::remove_file(&encode_path);
        match remove_error {
            Ok(file) => file,
            Err(error) => {
                print(
                    Verbosity::ERROR,
                    From::Content,
                    called_from,
                    format!("Target for removal: {}", &encode_path),
                );
                panic!("Problem removing the file: {:?}", error);
            }
        };

        self.status_completed = true;
    }
}