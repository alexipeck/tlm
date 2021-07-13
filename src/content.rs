use crate::designation::Designation;
use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::VecDeque;
use std::process::Command;

use postgres_types::{ToSql, FromSql};

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static JOB_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);



#[derive(Clone, Debug)]
pub struct Job {
    pub uid: usize,
    pub source_path: PathBuf,
    pub encode_path: PathBuf,
    pub encode_string: Vec<String>,
    pub reserved_by: Option<String>,
    pub underway_status: bool,
    pub completed_status: bool,
}

impl Job {
    //maybe best to use a generic string
    pub fn new(source_path: PathBuf, encode_string: Vec<String>) -> Job {
        Job {
            uid: JOB_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            source_path: source_path.clone(),
            encode_path: Content::generate_encode_path_from_pathbuf(source_path),
            encode_string: encode_string,
            reserved_by: None,
            underway_status: false,
            completed_status: false,
        }
    }

    pub fn print(&self, called_from: &str) {
        
        crate::print::print(crate::print::Verbosity::INFO, called_from, Content::get_filename_from_pathbuf(self.source_path.clone()));
    }

    pub fn reserve(&mut self, operator: String) {
        self.reserved_by = Some(operator);
    }

    pub fn encode(&self) {
        println!("Encoding file \'{}\'", Content::get_filename_from_pathbuf(self.source_path.clone()));
        
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
        //println!("{}", String::from_utf8_lossy(&buffer.stderr).to_string());
    }

    pub fn handle(&mut self, operator: String) {
        println!("CP1");
        self.reserve(operator);
        self.underway_status = true;
        
        self.encode();

        let source_path = self.source_path.to_string_lossy().to_string();
        let encode_path = self.encode_path.to_string_lossy().to_string();

        let copy_error = std::fs::copy(&encode_path, &source_path);
        match copy_error {
            Ok(file) => file,
            Err(error) => {
                println!("ERROR: \nSource: {}\nDestination: {}", &source_path, &encode_path);
                panic!("Problem copying the file: {:?}", error);
            }
        };
        let remove_error = std::fs::remove_file(&encode_path);
        match remove_error {
            Ok(file) => file,
            Err(error) => {
                println!("ERROR: \nTarget for removal: {}", &encode_path);
                panic!("Problem removing the file: {:?}", error);
            }
        };
        
        self.completed_status = true;
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

//generic content container, focus on video
#[derive(Clone, Debug)]
pub struct Content {
    pub uid: usize,
    pub full_path: PathBuf,
    //pub temp_encode_path: Option<PathBuf>,
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
        return self.get_full_path_with_suffix("_encodeH4U8".to_string()).to_string_lossy().to_string();
    }

    pub fn create_job(&mut self, encode_string: Vec<String>) -> Job {        
        return Job::new(self.full_path.clone(), encode_string);
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

    /* pub fn set_temp_encode_path(&mut self, pathbuf: std::path::PathBuf) {
        self.temp_encode_path = Some(pathbuf);
    } */

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
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
        return self.get_full_path_with_suffix(suffix).to_string_lossy().to_string();
    }

    pub fn get_full_path_with_suffix_from_pathbuf(pathbuf: PathBuf, suffix: String) -> PathBuf {
        //C:\Users\Alexi Peck\Desktop\tlm\test_files\episodes\Test Show\Season 3\Test Show - S03E02 - tf8.mp4\_encodeH4U8\mp4
        //.push(self.full_path.extension().unwrap())
        //bad way of doing it
        let new_filename = format!("{}{}.{}", 
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
        let new_filename = format!("{}{}.{}", 
            self.full_path.file_stem().unwrap().to_string_lossy().to_string(), 
            &suffix,
            self.full_path.extension().unwrap().to_string_lossy().to_string(),
        );
        return self.full_path.parent().unwrap().join(new_filename);
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
