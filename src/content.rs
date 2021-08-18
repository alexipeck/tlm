use crate::{
    database::execution::get_by_query,
    designation::{convert_i32_to_designation, Designation},
    job::Job,
    print::{print, From, Verbosity},
    show::Show,
    utility::Utility,
};
use std::{
    collections::HashSet,
    path::PathBuf,
};
use tokio_postgres::Row;
use regex::Regex;

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
    //generic
    pub content_uid: Option<usize>,
    pub full_path: PathBuf,
    pub designation: Designation,
    //pub job_queue: VecDeque<Job>,
    pub hash: Option<u64>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump

    //episode
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, Vec<usize>)>,
}

impl Content {
    //needs to be able to be created from a pathbuf or pulled from the database
    pub fn new(raw_filepath: &PathBuf, working_shows: &mut Vec<Show>, utility: Utility) -> Content {
        let utility = utility.clone_and_add_location("new");

        let mut content = Content {
            full_path: raw_filepath.clone(),
            designation: Designation::Generic,
            content_uid: None,
            hash: None,

            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill(working_shows, utility.clone());
        return content;
    }

    pub fn from_row(row: Row, working_shows: &mut Vec<Show>, utility: Utility) -> Content {
        let mut utility = utility.clone_and_add_location("from_row");

        utility.start_timer(0);
        let content_uid_temp: i32 = row.get(0);
        let full_path_temp: String = row.get(1);
        let designation_temp: i32 = row.get(2);

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let mut content = Content {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            content_uid: Some(content_uid_temp as usize),
            hash: None,

            //truly optional
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "from_row: initial content fill",
            2,
            utility.clone(),
        );

        utility.start_timer(1);
        content.designate_and_fill(working_shows, utility.clone());
        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "from_row: designate_and_fill",
            2,
            utility.clone(),
        );

        return content;
    }

    pub fn get_all_contents(working_shows: &mut Vec<Show>, utility: Utility) -> Vec<Content> {
        let mut utility = utility.clone_and_add_location("get_all_contents");
        utility.start_timer(0);

        utility.start_timer(1);
        let raw_content = get_by_query(
            r"SELECT content_uid, full_path, designation FROM content",
            utility.clone(),
        );
        utility.save_timing(1, utility.clone());

        let mut content: Vec<Content> = Vec::new();
        let mut counter = 2;
        for row in raw_content {
            utility.start_timer(counter);
            content.push(Content::from_row(row, working_shows, utility.clone()));
            utility.save_timing(counter, utility.clone());

            counter += 1;
        }

        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "read in content",
            0,
            utility.clone(),
        );
        for i in 2..utility.timers.len() {
            utility.print_timer_from_stage_and_task_from_saved(
                i,
                "startup",
                &format!("creating content from row: {}", i),
                1,
                utility.clone(),
            );
        }

        return content;
    }

    pub fn filename_from_row_as_pathbuf(row: Row) -> PathBuf {
        let temp: String = row.get(0);
        return PathBuf::from(temp);
    }

    pub fn get_all_filenames_as_hashset_from_contents(
        contents: Vec<Content>,
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility =
            utility.clone_and_add_location("get_all_filenames_as_hashset_from_contents");
        utility.start_timer(0);

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
                utility.get_timer_ms(0, utility.clone()),
            ),
            0,
        );

        return hashset;
    }

    pub fn get_all_filenames_as_hashset(utility: Utility) -> HashSet<PathBuf> {
        let utility = utility.clone_and_add_location("get_all_filenames_as_hashset");

        let mut hashset = HashSet::new();
        for row in get_by_query(r"SELECT full_path FROM content", utility.clone()) {
            hashset.insert(Content::filename_from_row_as_pathbuf(row));
        }
        return hashset;
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

    pub fn seperate_season_episode(&mut self, episode: &mut bool) -> Option<(usize, Vec<usize>)> {
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

        let mut season_episode_iter = episode_string.split('E');
        let season_temp = season_episode_iter
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut episodes: Vec<usize> = Vec::new();
        for episode in season_episode_iter.next().unwrap().split('-') {
            episodes.push(episode.parse::<usize>().unwrap());
        }

        return Some((season_temp, episodes));
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

    pub fn get_episode_string(&self) -> String {
        if self.show_season_episode.is_some() {
            let episode = self.show_season_episode.clone().unwrap().1;
            if episode.len() < 1 {
                panic!("Bad boy, you fucked up. There was less than 1 episode in the thingo");
            } else {
                let mut prepare = String::new();
                let mut first: bool = true;
                for episode in episode {
                    if first {
                        prepare.push_str(&format!("{}", episode));
                        first = false;
                    } else {
                        prepare += &format!("_{}", episode);
                    }
                }
                return prepare;
            }
        } else {
            panic!("Bad boy, you fucked up. show_season_episode is_none");
        }
    }

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_and_add_location("print");

        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            print(
                Verbosity::DEBUG,
                From::DB,
                utility.clone(),
                format!(
                    "[content_uid:'{}'][designation:'{}'][full_path:'{}'][show_uid:'{}'][show_title:'{}'][season:'{}'][episode:'{}']",
                    self.content_uid.unwrap(),
                    self.designation as i32,
                    self.get_full_path(),
                    self.show_uid.unwrap(),
                    self.show_title.clone().unwrap(),
                    self.get_episode_string(),
                    //self.show_season_episode.clone().unwrap().1[0],//asdf;
                    self.get_episode_string(),
                ),
            0,
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
                self.content_uid.unwrap(),
                self.designation as i32,
                self.get_full_path(),
            ),
            0,
        );
    }

    pub fn get_parent_directory_from_pathbuf_as_string(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn set_uid(&mut self, content_uid: usize) {
        self.content_uid = Some(content_uid);
    }

    pub fn set_show_uid(&mut self, show_uid: usize) {
        self.show_uid = Some(show_uid);
    }

    pub fn content_is_episode(&self) -> bool {
        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            return true;
        }
        //print(Verbosity::INFO, From::Main, traceback, format!("exists: [show_uid: {}][show_title: {}][show_season_episode: {}]", self.show_uid.is_some(), self.show_title.is_some(), self.show_season_episode.is_some()));
        return false;
    }

    pub fn designate_and_fill(&mut self, working_shows: &mut Vec<Show>, utility: Utility) {
        let mut utility = utility.clone_and_add_location("designate_and_fill");

        utility.start_timer(0);
        let mut episode = false;
        let show_season_episode_temp = self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic
        utility.print_timer_from_stage_and_task(
            0,
            "startup",
            "separate out season and episode from filename",
            3,
            utility.clone(),
        );

        if episode {
            utility.start_timer(1);
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
            utility.print_timer_from_stage_and_task(
                1,
                "startup",
                "get show title from filename",
                3,
                utility.clone(),
            );

            utility.start_timer(2);
            self.show_season_episode = show_season_episode_temp;
            self.show_uid = Some(Show::ensure_show_exists(
                self.show_title.clone().unwrap(),
                working_shows,
                utility.clone(),
            ));
            utility.print_timer_from_stage_and_task(
                2,
                "startup",
                "set show_season_episode from temp, ensure_show_exists",
                3,
                utility.clone(),
            );
        } else {
            self.designation = Designation::Generic;
            self.show_title = None;
            self.show_season_episode = None;
        }
    }

    pub fn regenerate_from_pathbuf(
        &mut self,
        working_shows: &mut Vec<Show>,
        raw_filepath: &PathBuf,
        utility: Utility,
    ) {
        let utility = utility.clone_and_add_location("regenerate_from_pathbuf");

        let mut episode = false;
        self.seperate_season_episode(&mut episode); //TODO: This is checking if it's an episode because main is too cluttered right now to unweave the content and show logic

        if episode {
            self.designation = Designation::Episode;
        } else {
            self.designation = Designation::Generic;
        };
        self.full_path = raw_filepath.clone();

        self.designate_and_fill(working_shows, utility);
    }
}
