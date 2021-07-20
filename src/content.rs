use crate::designation::Designation;
use crate::print::{print, From, Verbosity};
use regex::Regex;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static JOB_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn print_content(verbosity: Verbosity, called_from: String) {}

/* #[derive(Clone, Debug)]
pub struct Reserve {
    status: (bool, bool),
    uid: usize,
    worker: String,
}

impl Reserve {
    pub fn new(uid: usize, worker: String) -> Reserve {
        Reserve {
            status: (false, false),
            uid: uid,
            worker: worker,
        }
    }
} */

#[derive(Clone, Debug)]
pub enum Task {
    Encode = 0,
    Copy = 1,
    Move = 2,
    Rename = 3,
    Reserve = 4,
    Delete = 5,
    Reencode = 6,
    Duplicate = 7,
}

pub fn convert_task_id_to_task(task_id: usize) -> Task {
    match task_id {
        0 => {
            return Task::Encode;
        }
        1 => {
            return Task::Copy;
        }
        2 => {
            return Task::Move;
        }
        3 => {
            return Task::Rename;
        }
        4 => {
            return Task::Reserve;
        }
        5 => {
            return Task::Delete;
        }
        6 => {
            return Task::Reencode;
        }
        7 => {
            return Task::Duplicate;
        }
        _ => {
            panic!("Not valid task ID");
        }
    }
}

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

pub fn re_strip(input: &String, expression: &str) -> Option<String> {
    let output = Regex::new(expression).unwrap().find(input);
    match output {
        None => return None,
        Some(val) => return Some(String::from(rem_first_char(val.as_str()))),
    }
}

fn rem_first_char(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}

fn get_os_slash() -> char {
    return if !cfg!(target_os = "windows") {
        '/'
    } else {
        '\\'
    };
}

//generic content container, focus on video
#[derive(Clone, Debug)] //, Insertable
                        //#[table_name="content"]
pub struct Content {
    pub uid: usize,
    pub full_path: PathBuf,
    pub designation: Designation,
    //pub job_queue: VecDeque<Job>,
    pub hash: Option<u64>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, usize)>,
}

impl Content {
    pub fn new(raw_filepath: &PathBuf) -> Content {
        let mut content = Content {
            full_path: raw_filepath.clone(),
            //temp_encode_path: None,
            designation: Designation::Generic,
            uid: EPISODE_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            hash: None,
            //job_queue: VecDeque::new(),

            //truly optional
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill();
        return content;
    }

    //no options currently
    pub fn generate_encode_string(&self) -> Vec<String> {
        return vec![
            "-i".to_string(),
            self.get_full_path(),
            "-c:v".to_string(),
            "libx265".to_string(),
            "-crf".to_string(),
            "25".to_string(),
            "-preset".to_string(),
            "slower".to_string(),
            "-profile:v".to_string(),
            "main".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-q:a".to_string(),
            "224k".to_string(),
            "-y".to_string(),
            self.generate_target_path(),
        ];
    }

    pub fn generate_target_path(&self) -> String {
        return self
            .get_full_path_with_suffix("_encodeH4U8".to_string())
            .to_string_lossy()
            .to_string();
    }

    pub fn create_job(&mut self) -> Job {
        return Job::new(self.full_path.clone(), self.generate_encode_string());
    }

    pub fn generate_encode_path_from_pathbuf(pathbuf: PathBuf) -> PathBuf {
        return Content::get_full_path_with_suffix_from_pathbuf(pathbuf, "_encodeH4U8".to_string());
    }

    pub fn seperate_season_episode(&mut self, episode: &mut bool) -> Option<(usize, usize)> {
        let episode_string: String;

        //Check if the regex caught a valid episode format
        match re_strip(&self.get_filename(), r"S[0-9]*E[0-9\-]*") {
            None => {
                *episode = false;
                return None;
            }
            Some(temp_string) => {
                *episode = true;
                episode_string = temp_string;
            }
        }

        let mut se_iter = episode_string.split('E');
        Some((
            se_iter.next().unwrap().parse::<usize>().unwrap(),
            se_iter.next().unwrap().parse::<usize>().unwrap(),
        ))
    }

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn get_filename(&self) -> String {
        return self
            .full_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }

    pub fn get_filename_woe(&self) -> String {
        return self
            .full_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
    }

    pub fn get_parent_directory_as_string(&self) -> String {
        return self
            .full_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
    }

    pub fn get_full_path_with_suffix_as_string(&self, suffix: String) -> String {
        return self
            .get_full_path_with_suffix(suffix)
            .to_string_lossy()
            .to_string();
    }

    pub fn get_full_path_with_suffix_from_pathbuf(pathbuf: PathBuf, suffix: String) -> PathBuf {
        //C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\Test Show\Season 3\Test Show - S03E02 - tf8.mp4\_encodeH4U8\mp4
        //.push(self.full_path.extension().unwrap())
        //bad way of doing it
        let new_filename = format!(
            "{}{}.{}",
            pathbuf.file_stem().unwrap().to_string_lossy().to_string(),
            &suffix,
            pathbuf.extension().unwrap().to_string_lossy().to_string(),
        );
        return pathbuf.parent().unwrap().join(new_filename);
    }

    pub fn get_full_path_with_suffix(&self, suffix: String) -> PathBuf {
        //C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\Test Show\Season 3\Test Show - S03E02 - tf8.mp4\_encodeH4U8\mp4
        //.push(self.full_path.extension().unwrap())
        //bad way of doing it
        let new_filename = format!(
            "{}{}.{}",
            self.full_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            &suffix,
            self.full_path
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
        return self.full_path.parent().unwrap().join(new_filename);
    }

    pub fn get_parent_directory_from_pathbuf_as_string(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn set_show_uid(&mut self, show_uid: usize) {
        self.show_uid = Some(show_uid);
    }

    pub fn designate_and_fill(&mut self) {
        let mut episode = false;
        let show_season_episode_conditional = self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic
        if episode {
            self.designation = Designation::Episode;
            for section in String::from(
                self.full_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_string_lossy(),
            )
            .split(get_os_slash())
            .rev()
            {
                self.show_title = Some(String::from(section));
                break;
            }

            self.show_season_episode = show_season_episode_conditional;
            self.show_uid = None;
        } else {
            self.designation = Designation::Generic;
            self.show_title = None;
            self.show_season_episode = None;
        }
    }

    pub fn moved(&mut self, new_full_path: &PathBuf) {
        self.full_path = new_full_path.clone();
    }

    pub fn regenerate_from_pathbuf(&mut self, raw_filepath: &PathBuf) {
        let mut episode = false;
        self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic

        if episode {
            self.designation = Designation::Episode;
        } else {
            self.designation = Designation::Generic;
        };
        self.full_path = raw_filepath.clone();

        //designation, show_title, show_season_episode
        self.designate_and_fill();
    }
}
