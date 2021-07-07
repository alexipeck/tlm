use regex::Regex;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SHOW_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq)]
pub enum Designation {
    Generic,
    Episode,
    Movie,
}

pub struct Season {
    pub number: u8,
    pub episodes: Vec<Content>,
}

pub struct Show {
    pub uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}


pub struct Shows {
    pub shows: Vec<Show>,
}
impl Shows {
    fn find_index_by_uid(&self, uid: usize) -> Option<usize> {//if !is_none(show_uid)
        return self.shows.iter().position(|show| show.uid == uid);
    }
    
    fn is_episode(content: Content) -> bool {
        if content.designation == Designation::Episode {
            return true;
        } else {
            return false;
        }
    }

    pub fn ensure_exists_by_title(&mut self, title: String) -> usize {
        let exists = false;
        let index: usize = 0;
        for show in self.shows {
            if show.title == title {
                exists = true;
                return index;
            }
            index += 1;
        }
        self.shows.push();
        uid = SHOW_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
    }

    //will overwrite data
    fn insert_in_order(&mut self, content: Content) {
        let show_index = self.find_index_by_uid(content.uid).unwrap();
        let se_temp = content.show_season_episode.clone().unwrap();
        let season_index = se_temp.0.parse::<usize>().unwrap();
        let episode_index = se_temp.1.parse::<usize>().unwrap();
        self.shows[show_index].seasons[season_index].episodes[episode_index] = content;
    }

    pub fn add_episode(&mut self, episode: Content) {
        if episode.designation == Designation::Episode {
            //shows.push()
        }
        //handle error
    }


    //insert show

    //exists

    //pub collect season

    //pub collect show



}

//generic content container, focus on video
#[derive(Clone)]
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
    pub show_season_episode: Option<(String, String)>,
}

impl Content {
    pub fn new(raw_filepath: &PathBuf) -> Content {
        //prepare filename
        let filename = String::from(raw_filepath.file_name().unwrap().to_string_lossy());

        let mut episode = false;
        seperate_season_episode(&filename, &mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic

        //prepare filename without extension
        let filename_woe = String::from(raw_filepath.file_stem().unwrap().to_string_lossy());

        //parent directory
        let parent_directory = String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/");

        let extension = String::from(raw_filepath.extension().unwrap().to_string_lossy());

        let designation: Designation;
        if episode {
            designation = Designation::Episode;
        } else {
            designation = Designation::Generic;
        }
        Content {
            full_path: raw_filepath.clone(),
            designation: designation,
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
        }
    }

    pub fn designate_and_fill(&mut self) {
        let mut episode = false;
        let show_season_episode_conditional = seperate_season_episode(&self.filename, &mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic
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
        } else {
            self.designation = Designation::Generic;
            self.show_title = None;
            self.show_season_episode = None;
        }
    }

    pub fn moved(&mut self, raw_filepath: &PathBuf) {
        self.parent_directory = String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/");
        self.full_path = raw_filepath.clone();
    }

    pub fn regenerate(&mut self, raw_filepath: &PathBuf) {
        let filename = String::from(raw_filepath.file_name().unwrap().to_string_lossy());

        let mut episode = false;
        seperate_season_episode(&filename, &mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic


        self.extension = String::from(raw_filepath.extension().unwrap().to_string_lossy());

        if episode {
            self.designation = Designation::Episode;
        } else {
            self.designation = Designation::Generic;
        };
        self.full_path = raw_filepath.clone();
        self.parent_directory = String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/");
        self.filename = filename;
        self.filename_woe = String::from(raw_filepath.file_stem().unwrap().to_string_lossy());
        self.extension = String::from(raw_filepath.extension().unwrap().to_string_lossy());

        //designation, show_title, show_season_episode
        self.designate_and_fill();
    }
}

fn seperate_season_episode(filename: &String, episode: &mut bool) -> Option<(String, String)> {
    let temp = re_strip(filename, r"S[0-9]*E[0-9\-]*");
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
        se_iter.next().unwrap().to_string(),
        se_iter.next().unwrap().to_string(),
    ))
}

fn re_strip(input: &String, expression: &str) -> Option<String> {
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
