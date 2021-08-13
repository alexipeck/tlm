use crate::{
    content::Content,
    print::{print, From, Verbosity},
    task::Task,
    utility::Utility,
};
use std::{
    collections::VecDeque,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
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

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print");
        print(
            Verbosity::INFO,
            From::Content,
            utility,
            Content::get_filename_from_pathbuf(self.source_path.clone()),
            0,
        );
    }

    pub fn encode(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("encode");
        print(
            Verbosity::INFO,
            From::Job,
            utility,
            format!(
                "Encoding file \'{}\'",
                Content::get_filename_from_pathbuf(self.source_path.clone())
            ),
            0,
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

    pub fn handle(&mut self, worker: (usize, String), utility: Utility) {
        let utility = utility.clone_and_add_location("handle");
        print(
            Verbosity::INFO,
            From::Job,
            utility.clone(),
            format!("starting encoding job UID#: {} by {}", self.uid, worker.1),
            0,
        );
        self.encode(utility.clone());
        print(
            Verbosity::INFO,
            From::Job,
            utility.clone(),
            format!("completed encoding job UID#: {}", self.uid),
            0,
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
                    utility,
                    format!("Source: {}\nDestination: {}", &source_path, &encode_path),
                    0,
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
                    utility,
                    format!("Target for removal: {}", &encode_path),
                    0,
                );
                panic!("Problem removing the file: {:?}", error);
            }
        };

        self.status_completed = true;
    }
}
