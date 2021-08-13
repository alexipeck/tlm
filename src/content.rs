use crate::{
    database::ensure::ensure_show_exists,
    database::execution::get_by_query,
    designation::{convert_i32_to_designation, Designation},
    job::Job,
    print::{print, From, Verbosity},
    utility::Utility,
};
use regex::Regex;
use std::{
    collections::HashSet,
    time::Instant,
    sync::atomic::{AtomicUsize, Ordering},
    path::PathBuf,
};
use tokio_postgres::Row;

static EPISODE_UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

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

fn get_os_slash() -> char {
    return if !cfg!(target_os = "windows") {
        '/'
    } else {
        '\\'
    };
}

//generic content container, focus on video
#[derive(Clone, Debug)]
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
    //needs to be able to be created from a pathbuf or pulled from the database
    pub fn new(raw_filepath: &PathBuf, utility: Utility) -> Content {
        let utility = utility.clone_and_add_location("new");

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
        content.designate_and_fill(utility);
        return content;
    }

    pub fn from_row(row: Row, utility: Utility) -> Content {
        let utility = utility.clone_and_add_location("from_row");

        let content_uid_temp: i32 = row.get(0);
        let full_path_temp: String = row.get(1);
        let designation_temp: i32 = row.get(2);

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let mut content = Content {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            uid: content_uid_temp as usize,
            hash: None,

            //truly optional
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill(utility);

        return content;
    }

    /* pub fn get_contents_by_filter(filter: Filter) -> Vec<Content> {

    } */

    pub fn get_all_contents(utility: Utility) -> Vec<Content> {
        let mut utility = utility.clone_and_add_location("get_all_contents");
        utility.start_timer();

        let mut contents: Vec<Content> = Vec::new();
        for row in get_by_query(
            r"SELECT content_uid, full_path, designation FROM content",
            utility.clone(),
        ) {
            contents.push(Content::from_row(row, utility.clone()));
        }

        print(
            Verbosity::INFO,
            From::Main,
            utility.clone(),
            format!(
                "startup: read in 'content' took: {}ms",
                utility.get_timer_ms(),
            ),
        );
        
        return contents;
    }

    pub fn filename_from_row_as_pathbuf(row: Row, utility: Utility) -> PathBuf {
        let utility = utility.clone_and_add_location("filename_from_row_as_pathbuf");

        let temp: String = row.get(0);
        return PathBuf::from(temp);
    }

    pub fn get_all_filenames_as_hashset_from_contents(
        contents: Vec<Content>,
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility = utility.clone_and_add_location("get_all_filenames_as_hashset_from_contents");
        utility.start_timer();

        /*
         * logic
         */
        let mut hashset = HashSet::new();
        for content in contents {
            hashset.insert(content.full_path);
        }

        print(
            Verbosity::INFO,
            From::Main,
            utility.clone(),
            format!(
                "startup: read in 'existing files hashset' took: {}ms",
                utility.get_timer_ms(),
            ),
        );

        return hashset;
        //////////
    }

    pub fn get_all_filenames_as_hashset(utility: Utility) -> HashSet<PathBuf> {
        let utility = utility.clone_and_add_location("get_all_filenames_as_hashset");

        /*
         * logic
         */
        let mut hashset = HashSet::new();
        for row in get_by_query(r"SELECT full_path FROM content", utility.clone()) {
            hashset.insert(Content::filename_from_row_as_pathbuf(
                row,
                utility.clone(),
            ));
        }
        return hashset;
        //////////
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

    pub fn get_full_path_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.to_str().unwrap().to_string();
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

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print");

        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            let season_episode = self.show_season_episode.unwrap();
            print(
                Verbosity::DEBUG,
                From::DB,
                utility.clone(),
                format!(
                    "[content_uid:'{}'][designation:'{}'][full_path:'{}'][show_uid:'{}'][show_title:'{}'][season:'{}'][episode:'{}']",
                    self.uid,
                    self.designation as i32,
                    self.get_full_path(),
                    self.show_uid.unwrap(),
                    self.show_title.clone().unwrap(),
                    season_episode.0,
                    season_episode.1
                ),
            );
        } else {
            self.print_simple(utility);
        }
    }

    fn print_simple(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print_simple");

        print(
            Verbosity::DEBUG,
            From::DB,
            utility.clone(),
            format!(
                "[content_uid:'{}'][designation:'{}'][full_path:'{}']",
                self.uid,
                self.designation as i32,
                self.get_full_path(),
            ),
        );
    }

    pub fn get_parent_directory_from_pathbuf_as_string(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn set_uid(&mut self, uid: usize) {
        self.uid = uid;
    }

    pub fn set_show_uid(&mut self, show_uid: usize) {
        self.show_uid = Some(show_uid);
    }

    pub fn content_is_episode(&self, utility: Utility) -> bool {
        let utility = utility.clone_and_add_location("content_is_episode");

        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            return true;
        }
        //print(Verbosity::INFO, From::Main, traceback, format!("exists: [show_uid: {}][show_title: {}][show_season_episode: {}]", self.show_uid.is_some(), self.show_title.is_some(), self.show_season_episode.is_some()));
        return false;
    }

    pub fn designate_and_fill(&mut self, utility: Utility) {
        let utility = utility.clone_and_add_location("designate_and_fill");

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
            //check if show title already exists in the db, if not, create show and return uid
            //asd;
            self.show_uid = ensure_show_exists(self.show_title.clone().unwrap(), utility);
        } else {
            self.designation = Designation::Generic;
            self.show_title = None;
            self.show_season_episode = None;
        }
    }

    pub fn regenerate_from_pathbuf(&mut self, raw_filepath: &PathBuf, utility: Utility) {
        let utility = utility.clone_and_add_location("regenerate_from_pathbuf");

        let mut episode = false;
        self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic

        if episode {
            self.designation = Designation::Episode;
        } else {
            self.designation = Designation::Generic;
        };
        self.full_path = raw_filepath.clone();

        //designation, show_title, show_season_episode
        self.designate_and_fill(utility);
    }
}
