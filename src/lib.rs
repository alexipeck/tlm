use regex::Regex;
use std::collections::VecDeque;
use std::fs;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::process::Command; //borrow::Cow, thread::current,
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use twox_hash::xxh3;
use walkdir::WalkDir;

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static SHOW_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq)]
pub enum Designation {
    Generic,
    Episode,
    Movie,
}

pub struct Season {
    pub number: usize,
    pub episodes: Vec<Content>,
}

impl Season {
    pub fn new(number: usize) -> Season {
        let episodes = Vec::new();
        Season {
            number: number,
            episodes: episodes,
        }
    }

    pub fn insert_in_order(&mut self, content: Content) {
        //not in order, but that's fine for now, just doing member stuff
        self.episodes.push(content);
    }
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

//Return true in string contains any substring from Vector
fn str_contains_strs(input_str: &str, substrings: &Vec<&str>) -> bool {
    for substring in substrings {
        if String::from(input_str).contains(substring) {
            return true;
        }
    }
    false
}

pub fn import_files(
    file_paths: &mut Vec<PathBuf>,
    directories: &Vec<String>,
    allowed_extensions: &Vec<&str>,
    ignored_paths: &Vec<&str>,
) {
    //import all files in tracked root directories
    for directory in directories {
        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            if str_contains_strs(entry.path().to_str().unwrap(), ignored_paths) {
                break;
            }

            if entry.path().is_file() {
                if allowed_extensions.contains(&entry.path().extension().unwrap().to_str().unwrap())
                {
                    if !directory.contains("_encodeH4U8") {
                        file_paths.push(entry.into_path());
                    }
                }
            }
        }
    }
}

fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
}

pub fn get_os_slash() -> String {
    return if !cfg!(target_os = "windows") {
        '/'.to_string()
    } else {
        '\\'.to_string()
    };
}

pub fn rename(source_string: &String, target_string: &String) -> std::io::Result<()> {
    std::fs::copy(&source_string, &target_string)?;
    std::fs::remove_file(&source_string)?;
    Ok(())
}

pub struct Queue {
    pub priority_queue: VecDeque<Content>,
    pub main_queue: VecDeque<Content>,
}

pub enum QueueType {
    MainQueue,
    PriorityQueue,
    All,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            priority_queue: VecDeque::new(),
            main_queue: VecDeque::new(),
        }
    }

    pub fn print(&mut self) {
        for content in &self.priority_queue {
            println!("{}", content.get_full_path());
        }
    
        for content in &self.main_queue {
            println!("{}", content.get_full_path());
        }
    }

    pub fn get_index_by_uid_in_queue(&mut self, uid: usize, queue_type: QueueType) -> Option<usize> {
        match queue_type {
            QueueType::PriorityQueue => {
                for (pos, content) in self.priority_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            },
            QueueType::MainQueue => {
                for (pos, content) in self.main_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            },
            QueueType::All => {
                for (pos, content) in self.priority_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }

                for (pos, content) in self.main_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            },
        }
        
        return None;
    }

    pub fn get_index_by_uid(&self, uid: usize) -> Option<(usize, QueueType)> {
        for (pos, content) in self.priority_queue.iter().enumerate() {
            if content.uid == uid {
                return Some((pos, QueueType::PriorityQueue));
            }
        }

        for (pos, content) in self.main_queue.iter().enumerate() {
            if content.uid == uid {
                return Some((pos, QueueType::PriorityQueue));
            }
        }

        return None;
    }

    pub fn get_full_queue_length(&mut self) -> usize {
        return self.priority_queue.len() + self.main_queue.len();
    }

    pub fn encode_and_rename_next_unreserved(&mut self, operator: String) {
        let mut working_content: Option<&mut Content> = None;
        for content in &mut self.priority_queue {
            if content.reserved_by == None {
                working_content = Some(content);
            }
        }
    
        for content in &mut self.main_queue {
            if content.reserved_by == None {
                working_content = Some(content);
            }
        }

        match working_content {
            None => {
                //nothing available for encode
            }
            Some(working_content) => {
                working_content.reserve(operator);
                working_content.encode();
                
            }
        }
    }

    pub fn get_next_unreserved(&mut self, operator: String) -> Option<usize> {
        for content in &mut self.priority_queue {
            if content.reserved_by == None {
                content.reserve(operator);
                return Some(content.uid);
            }
        }
    
        for content in &mut self.main_queue {
            if content.reserved_by == None {
                content.reserve(operator);
                return Some(content.uid);
            }
        }
        
        return None;
    }

    pub fn exists_pmq(&self, uid: usize) -> bool {
        return self.exists_pq(uid) || self.exists_mq(uid);
    }

    pub fn exists_pq(&self, uid: usize) -> bool {
        for content in &self.priority_queue {
            if content.reserved_by == None {
                return true;
            }
        }
        return false;
    }

    pub fn exists_mq(&self, uid: usize) -> bool {
        for content in &self.main_queue {
            if content.reserved_by == None {
                return true;
            }
        }
        return false;
    }

    pub fn get_content_by_uid(&self, uid: usize) -> Option<(Content, QueueType)> {
        for content in &self.priority_queue {
            if content.uid == uid {
                return Some((content.clone(), QueueType::PriorityQueue));
            }
        }

        for content in &self.main_queue {
            if content.uid == uid {
                return Some((content.clone(), QueueType::MainQueue));
            }
        }

        return None;
    }

    fn remove_from_queue_by_uid(&mut self, uid: usize, queue_type: QueueType) -> Option<Content> {
        let temp;
        match queue_type {
            QueueType::PriorityQueue => {
                temp = self.get_index_by_uid_in_queue(uid, QueueType::PriorityQueue);
                if temp.is_some() {
                    self.priority_queue.remove(temp.unwrap());
                }
            },
            QueueType::MainQueue => {
                temp = self.get_index_by_uid_in_queue(uid, QueueType::MainQueue);
                self.priority_queue.remove(temp.unwrap());
            },
            QueueType::All => {
                let pq = self.get_index_by_uid_in_queue(uid, QueueType::PriorityQueue);
                if pq.is_some() {
                    self.priority_queue.remove(pq.unwrap());
                }
                let mq = self.get_index_by_uid_in_queue(uid, QueueType::MainQueue);
                if mq.is_some() {
                    self.priority_queue.remove(mq.unwrap());
                }
            },
        }
        return None;
    }

    pub fn prioritise_existing_encode(&mut self, uid: usize) {
        let main_queue = self.exists_mq(uid);
        let priority_queue = self.exists_pq(uid);
        
        if main_queue && !priority_queue {
            match self.get_content_by_uid(uid) {
                None => {
                    //nothing to do
                }
                Some((content, location)) => {
                    if content.reserved_by.is_none() {
                        let content = self.remove_from_queue_by_uid(uid, location);
                        if content.is_some() {
                            self.priority_queue.push_back(content.unwrap());
                        }
                    }
                }
            }
        }
    }

    pub fn prioritise_content_by_content(&mut self, content: Content) {
        let mut exists = false;
        if !self.exists_pq(content.uid) {
            self.priority_queue.push_back(content);
        }
    }

    /* pub fn prioritise_content_by_uids(&mut self, uids: Vec<usize>) {
        while uids.len() > 0 {
            let mut exists = false;
            let current_uid = uids.
            if !exists
        }
        
            
            if found {
                self.priority_queue.push(self.main_queue.remove(index));
            }
    } */
}

pub struct Show {
    pub uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

impl Show {
    pub fn new(uid: usize, title: String) -> Show {
        Show {
            uid: uid,
            title: title,
            seasons: Vec::new(),
        }
    }
}

//Season
impl Index<usize> for Show {
    type Output = Season;
    fn index(&self, season: usize) -> &Season {
        &self.seasons[season]
    }
}

impl IndexMut<usize> for Show {
    fn index_mut<'a>(&'a mut self, season: usize) -> &'a mut Season {
        &mut self.seasons[season]
    }
}

//Episode
impl Index<(usize, usize)> for Show {
    type Output = Content;
    fn index(&self, (season, episode): (usize, usize)) -> &Content {
        &self.seasons[season].episodes[episode]
    }
}

impl IndexMut<(usize, usize)> for Show {
    fn index_mut<'a>(&'a mut self, (season, episode): (usize, usize)) -> &'a mut Content {
        &mut self.seasons[season].episodes[episode]
    }
}

//Show
impl Index<usize> for Shows {
    type Output = Show;
    fn index(&self, show: usize) -> &Show {
        &self.shows[show]
    }
}

impl IndexMut<usize> for Shows {
    fn index_mut<'a>(&'a mut self, show: usize) -> &'a mut Show {
        &mut self.shows[show]
    }
}

pub struct Shows {
    pub shows: Vec<Show>,
}

impl Shows {
    fn find_index_by_uid(&self, uid: usize) -> Option<usize> {
        //if !is_none(show_uid)
        return self.shows.iter().position(|show| show.uid == uid);
    }

    pub fn new() -> Shows {
        Shows { shows: Vec::new() }
    }

    fn ensure_season_exists_by_show_index_and_season_number(
        &mut self,
        show_index: usize,
        season_number: usize,
    ) {
        for season in &mut self.shows[show_index].seasons {
            if season.number == season_number {
                break;
            }
        }
        self[show_index].seasons.push(Season::new(season_number));
    }

    //returns (uid, index)
    pub fn ensure_show_exists_by_title(&mut self, title: String) -> (usize, usize) {
        let mut index: usize = 0;
        for show in &self.shows {
            if show.title == title {
                return (show.uid, index);
            }
            index += 1;
        }
        let uid = SHOW_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        self.shows.push(Show::new(uid, title));
        return (uid, index);
    }

    //not actually in order
    fn insert_in_order(
        &mut self,
        show_index: usize,
        //season_number: usize,
        //_episode_number: usize,
        content: Content,
    ) {
        //remember episode_number
        //let mut inserted = false;
        for season in &mut self[show_index].seasons {
            if season.number == content.show_season_episode.unwrap().0 {
                //let mut index: usize = 0;

                season.insert_in_order(content.clone());
                /* for episode in &mut season.episodes {
                    let current = episode.show_season_episode.clone().unwrap().1.parse::<usize>().unwrap();
                    if index + 1 <= season.episodes.len() {
                        let next = season.episodes[index + 1].show_season_episode.clone().unwrap().1.parse::<usize>().unwrap();

                        if current < episode_number && next > episode_number {
                            season.episodes.insert(index, content);
                            inserted = true;
                        }
                    }
                    index += 1;
                }
                if !inserted {
                    season.episodes.push(content);
                } */
            }
        }
    }

    //will overwrite data
    pub fn add_episode(&mut self, content: Content) {
        let show_index = self
            .ensure_show_exists_by_title(content.show_title.clone().unwrap())
            .1;
        let se_temp = content.show_season_episode.clone().unwrap();
        let season_number = se_temp.0;
        let episode_number = se_temp.1;
        self.ensure_season_exists_by_show_index_and_season_number(show_index, season_number);
        self.insert_in_order(show_index, content);
    }

    //insert show

    //exists

    //pub collect season

    //pub collect show

    pub fn print(&self) {
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    println!("{}", episode.filename_woe,);
                }
            }
        }
    }
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

    fn seperate_season_episode(&mut self, episode: &mut bool) -> Option<(usize, usize)> {
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

    fn get_filename_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    fn get_filename_woe_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.file_stem().unwrap().to_string_lossy().to_string();
    }

    fn get_parent_directory(&self) -> String {
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

    fn get_parent_directory_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    fn get_extension_from_pathbuf(pathbuf: &PathBuf) -> String {
        return pathbuf.extension().unwrap().to_string_lossy().to_string();
    }

    pub fn get_season_number(&mut self) -> usize {
        return self.show_season_episode.unwrap().0;
    }

    pub fn get_episode_number(&mut self) -> usize {
        return self.show_season_episode.unwrap().1;
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
