use crate::designation::Designation;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

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

//generic content container, focus on video
#[derive(Clone, Debug)]
pub struct Content {
    pub uid: usize,
    pub full_path: PathBuf,
    pub designation: Designation,
    pub parent_directory: String,
    pub filename: String,
    pub filename_woe: String,
    pub reserved_by: Option<String>,
    pub extension: String,

    pub hash: Option<u64>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, usize)>,
}

impl Content {
    pub fn new(raw_filepath: &PathBuf) -> Content {
        //prepare filename
        let filename = Content::get_filename_from_pathbuf(raw_filepath);

        //prepare filename without extension
        let filename_woe = Content::get_filename_woe_from_pathbuf(raw_filepath);

        //parent directory
        let parent_directory = Content::get_parent_directory_from_pathbuf(raw_filepath);

        let extension = Content::get_extension_from_pathbuf(raw_filepath);
        let mut content = Content {
            full_path: raw_filepath.clone(),
            designation: Designation::Generic,
            uid: EPISODE_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            parent_directory: parent_directory,
            filename: filename,
            filename_woe: filename_woe,
            reserved_by: None,
            hash: None,
            extension: extension,

            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill();
        return content;
    }

    pub fn seperate_season_episode(&mut self, episode: &mut bool) -> Option<(usize, usize)> {
        let temp = re_strip(&self.filename, r"S[0-9]*E[0-9\-]*");
        let episode_string: String;

        //Check if the regex caught a valid episode format
        match temp {
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

    pub fn reserve(&mut self, operator: String) {
        self.reserved_by = Some(operator);
    }

    pub fn encode(&mut self) -> String {
        let source = self.get_full_path();
        let encode_target = self.get_full_path_with_suffix("_encodeH4U8".to_string()); //want it to use the actual extension rather than just .mp4

        let encode_string: Vec<&str> = vec![
            "-i",
            &source,
            "-c:v",
            "libx265",
            "-crf",
            "25",
            "-preset",
            "slower",
            "-profile:v",
            "main",
            "-c:a",
            "aac",
            "-q:a",
            "224k",
            "-y",
            &encode_target,
        ];

        println!("Encoding file \'{}\'", self.get_filename());

        let buffer;
        if !cfg!(target_os = "windows") {
            //linux & friends
            buffer = Command::new("ffmpeg")
                .args(encode_string)
                .output()
                .expect("failed to execute process");
        } else {
            //windows
            buffer = Command::new("ffmpeg")
                .args(encode_string)
                .output()
                .expect("failed to execute process");
        }
        return String::from_utf8_lossy(&buffer.stdout).to_string();
    }

    pub fn get_full_path_specific_extension(&self, extension: String) -> String {
        return format!(
            "{}{}{}.{}",
            self.parent_directory,
            get_os_slash(),
            self.filename_woe,
            extension
        );
    }

    pub fn get_full_path_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_show_title_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
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

    pub fn get_filename_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn get_filename_woe_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.file_stem().unwrap().to_string_lossy().to_string();
    }

    pub fn get_parent_directory(&self) -> String {
        return self
            .full_path
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
    }

    pub fn get_full_path_with_suffix(&self, suffix: String) -> String {
        return format!(
            "{}{}{}{}.{}",
            self.get_parent_directory(),
            get_os_slash(),
            self.get_filename_woe(),
            suffix,
            self.extension
        );
    }

    pub fn get_parent_directory_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn get_extension_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.extension().unwrap().to_string_lossy().to_string();
    }

    pub fn get_season_number(&self) -> Option<usize> {
        return Some(self.show_season_episode.unwrap().0);
    }

    pub fn get_episode_number(&self) -> Option<usize> {
        return Some(self.show_season_episode.unwrap().1);
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

    pub fn moved(&mut self, raw_filepath: &PathBuf) {
        self.parent_directory =
            String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/");
        self.full_path = raw_filepath.clone();
    }

    pub fn regenerate(&mut self, raw_filepath: &PathBuf) {
        let filename = String::from(raw_filepath.file_name().unwrap().to_string_lossy());

        let mut episode = false;
        self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic

        self.extension = String::from(raw_filepath.extension().unwrap().to_string_lossy());

        if episode {
            self.designation = Designation::Episode;
        } else {
            self.designation = Designation::Generic;
        };
        self.full_path = raw_filepath.clone();
        self.parent_directory =
            String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/");
        self.filename = filename;
        self.filename_woe = String::from(raw_filepath.file_stem().unwrap().to_string_lossy());
        self.extension = String::from(raw_filepath.extension().unwrap().to_string_lossy());

        //designation, show_title, show_season_episode
        self.designate_and_fill();
    }
}
