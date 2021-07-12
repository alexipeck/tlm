use crate::designation::Designation;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::VecDeque;
use std::thread::current;

use print;

use postgres_types::{ToSql, FromSql};

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

fn get_os_slash() -> String {
    return if !cfg!(target_os = "windows") {
        '/'.to_string()
    } else {
        '\\'.to_string()
    };
}

/* impl Ord for Content {
    fn cmp(&self, other: &Self) -> Ordering {
        self.show_season_episode.parse::<usize>().unwrap().cmp(&other.show_season_episode.parse::<usize>().unwrap())
    }
}

impl PartialOrd for Content {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height
    }
} */

#[derive(Clone, Debug)]
pub struct Job {
    source_path: PathBuf,
    encode_path: PathBuf,
    encode_string: Vec<String>,
    pub reserved_by: Option<String>,
    underway_status: bool,
    completed_status: bool,
}

impl Job {
    //maybe best to use a generic string
    pub fn new(source_path: PathBuf, encode_string: Vec<String>) -> Job {
        Job {
            source_path: source_path.clone(),
            encode_path: Content::generate_encode_path_from_pathbuf(source_path),
            encode_string: encode_string,
            reserved_by: None,
            underway_status: false,
            completed_status: false,
        }
    }

    pub fn reserve(&mut self, operator: String) {
        self.reserved_by = Some(operator);
    }

    pub fn encode(self) {
        let buffer;
        if !cfg!(target_os = "windows") {
            //linux & friends
            buffer = Command::new("ffmpeg")
                .args(self.encode_string)
                .output()
                .expect("failed to execute process");
        } else {
            //windows
            buffer = Command::new("ffmpeg")
                .args(self.encode_string)
                .output()
                .expect("failed to execute process");
        }
        //println!("{}", String::from_utf8_lossy(&buffer.stdout).to_string());
    }

    pub fn handle(&mut self, operator: String) {
        self.reserve(operator);
        self.underway_status = true;
        
        self.encode();

        let error = std::fs::copy(self.source_path.to_string_lossy().to_string(), self.encode_path.to_string_lossy().to_string());
        std::fs::remove_file(self.source_path.to_string_lossy().to_string());


        self.completed_status = true;
    }
}

//generic content container, focus on video
#[derive(Clone, Debug)]
pub struct Content {
    pub uid: usize,
    pub full_path: PathBuf,
    //pub temp_encode_path: Option<PathBuf>,
    pub designation: Designation,
    pub job_queue: VecDeque<Job>,

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
            job_queue: VecDeque::new(),

            //truly optional
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill();
        return content;
    }

    fn generate_encode_path_from_pathbuf(pathbuf: PathBuf) -> PathBuf {
        return pathbuf.parent().unwrap().join(pathbuf.file_name().unwrap()).join(r"_encodeH4U8").join(pathbuf.extension().unwrap());
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

    pub fn run_encode(&mut self, operator: String) {
        //currently encodes first free option
        let mut current_job: Option<Job> = None;
        for job in &self.job_queue {
            if !(job.completed_status && job.underway_status) {
                current_job = Some(job.clone());
            }
        }

        if current_job.is_some() {
            println!("Encoding file \'{}\'", self.get_filename());
            current_job.unwrap().handle(operator);
        }
    }

    pub fn add_encode_to_job_queue(&mut self) {
        let encode_target = self.get_full_path_with_suffix_as_string("_encodeH4U8".to_string()); //want it to use the actual extension rather than just .mp4
        let encode_string: Vec<String> = vec![
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
            encode_target,
        ];
        self.job_queue.push_back(Job::new(self.full_path.clone(), encode_string.clone()));
    }

    /* pub fn set_temp_encode_path(&mut self, pathbuf: std::path::PathBuf) {
        self.temp_encode_path = Some(pathbuf);
    } */

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
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

    pub fn get_parent_directory(&self) -> String {
        return self
            .full_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
    }

    pub fn get_full_path_with_suffix_as_string(&self, suffix: String) -> String {
        return format!(
            "{}{}{}{}.{}",
            self.get_parent_directory(),
            get_os_slash(),
            self.get_filename_woe(),
            suffix,
            self.full_path.extension().unwrap().to_string_lossy().to_string(),
        );
    }

    pub fn get_full_path_with_suffix(&self, suffix: String) -> PathBuf {
        return self.full_path.parent().unwrap().join(self.full_path.file_name().unwrap()).join(suffix).join(self.full_path.extension().unwrap());
    }

    pub fn get_parent_directory_from_pathbuf(pathbuf: &PathBuf) -> String {
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
            .split('/')
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
