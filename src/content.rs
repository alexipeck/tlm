use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    designation::{convert_i32_to_designation, Designation},
    model::*,
    print::{print, From, Verbosity},
    profile::Profile,
    tv::Show,
    utility::Utility,
};
use diesel::pg::PgConnection;
use lazy_static::lazy_static;
use regex::Regex;

///Content contains fields for generic, episode, and movie data
/// this will obviously mean memory overhead. In future I think
/// we should split this into 3 types that would however mean
/// that the manager would require seperate vectors but I consider
/// that a non issue
#[derive(Clone, Debug, Queryable)]
pub struct Content {
    //generic
    pub content_uid: Option<usize>,
    pub full_path: PathBuf,
    pub designation: Designation,
    //pub job_queue: VecDeque<Job>,
    pub hash: Option<String>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump
    pub profile: Option<Profile>,
    //episode
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, Vec<usize>)>,
}

impl Content {
    //needs to be able to be created from a pathbuf or pulled from the database
    pub fn new(
        raw_filepath: &Path,
        working_shows: &mut Vec<Show>,
        utility: Utility,
        connection: &PgConnection,
    ) -> Self {
        let mut utility = utility.clone_add_location("new(Content)");

        let mut content = Content {
            full_path: raw_filepath.to_path_buf(),
            designation: Designation::Generic,
            content_uid: None,
            hash: None,
            profile: Profile::from_file(raw_filepath.to_path_buf()),

            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };
        content.designate_and_fill(working_shows, utility.clone(), connection);
        utility.print_function_timer();
        content
    }

    ///Hash the content file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        let hash = seahash::hash(&fs::read(self.full_path.to_str().unwrap()).unwrap());
        self.hash = Some(hash.to_string());
    }

    ///Create a new content from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_content_model(
        content_model: &ContentModel,
        working_shows: &mut Vec<Show>,
        utility: Utility,
        connection: &PgConnection,
    ) -> Content {
        let mut utility = utility.clone_add_location("from_row(Content)");

        let content_uid_temp: i32 = content_model.id;
        let full_path_temp: String = content_model.full_path.to_owned();
        let designation_temp: i32 = content_model.designation;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let mut content = Content {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            content_uid: Some(content_uid_temp as usize),
            hash: content_model.file_hash.to_owned(),
            profile: content_model.get_profile(),
            //truly optional
            show_title: None,
            show_season_episode: None,
            show_uid: None,
        };

        content.designate_and_fill(working_shows, utility.clone(), connection);
        utility.print_function_timer();

        content
    }

    pub fn get_all_filenames_as_hashset_from_content(
        contents: &[Content],
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        let mut hashset = HashSet::new();
        for content in contents {
            hashset.insert(content.full_path.clone());
        }

        utility.print_function_timer();
        hashset
    }

    ///Returns a vector of ffmpeg arguments for later execution
    /// This has no options currently
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

    ///Appends a fixed string to differentiate rendered files from original before overwrite
    /// I doubt this will stay as I think a temp directory would be more appropriate.
    /// This function returns that as a string for the ffmpeg arguments
    pub fn generate_target_path(&self) -> String {
        return self
            .get_full_path_with_suffix("_encodeH4U8".to_string())
            .to_string_lossy()
            .to_string();
    }

    /* pub fn create_job(&mut self) -> Job {
        return Job::new(self.full_path.clone(), self.generate_encode_string());
    } */

    ///Appends a fixed string to differentiate rendered files from original before overwrite
    /// I doubt this will stay as I think a temp directory would be more appropriate.
    /// This function returns that as a PathBuf
    pub fn generate_encode_path_from_pathbuf(pathbuf: PathBuf) -> PathBuf {
        Content::get_full_path_with_suffix_from_pathbuf(pathbuf, "_encodeH4U8".to_string())
    }

    pub fn seperate_season_episode(&mut self) -> Option<(usize, Vec<usize>)> {
        let episode_string: String;
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"S[0-9]*E[0-9\-]*").unwrap();
        }

        match REGEX.find(&self.get_filename()) {
            None => return None,
            Some(val) => episode_string = String::from(rem_first_char(val.as_str())),
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

        Some((season_temp, episodes))
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

    pub fn get_season_number(&self) -> usize {
        return self.show_season_episode.as_ref().unwrap().0;
    }

    pub fn get_show_title(&self, utility: Utility) -> String {
        let utility = utility.clone_add_location("get_show_title(Show)");

        if self.show_title.is_some() {
            self.show_title.clone().unwrap()
        } else {
            print(
                Verbosity::CRITICAL,
                From::Content,
                String::from("You called get_show_title on a content that didn't have an episode designation or was incorrectly created"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn get_show_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_show_uid(Show)");

        if self.show_uid.is_some() {
            self.show_uid.unwrap()
        } else {
            print(
                Verbosity::CRITICAL,
                From::Content,
                String::from("You called get_show_uid on a content that didn't have an episode designation or was incorrectly created"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn get_episode_string(&self) -> String {
        if self.show_season_episode.is_some() {
            let episode = self.show_season_episode.clone().unwrap().1;
            if episode.is_empty() {
                panic!("There was less than 1 episode in the thingo");
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
                prepare
            }
        } else {
            panic!("show_season_episode is_none");
        }
    }

    pub fn get_content_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_content_uid(Show)");

        if self.content_uid.is_some() {
            self.content_uid.unwrap()
        } else {
            print(
                Verbosity::CRITICAL,
                From::Content,
                String::from("get_content_uid was called on a content that hasn't been inserted into the db yet or hasn't been assigned a content_uid from the database correctly"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_add_location("print(Show)");

        //could realistically just check if it has an episode designation,
        //this just means that the content designation needs to be a guarantee rather than a designation
        if self.show_uid.is_some()
            && self.show_title.is_some()
            && self.show_season_episode.is_some()
        {
            print(
                Verbosity::DEBUG,
                From::Content,
                format!(
                    "[content_uid:'{:4}'][designation:'{}'][show_uid:'{:2}'][season:'{:2}'][episode:'{:2}'][full_path:'{}'][show_title:'{}']",
                    self.get_content_uid(utility.clone()),
                    self.designation as i32,
                    self.get_show_uid(utility.clone()),
                    self.get_season_number(),
                    self.get_episode_string(),
                    self.get_full_path(),
                    self.get_show_title(utility.clone()),
                ),
                utility.preferences.content_output_whitelisted,
                utility,
            );
        } else {
            print(
                Verbosity::DEBUG,
                From::DB,
                format!(
                    "[content_uid:'{}'][designation:'{}'][full_path:'{}']",
                    self.content_uid.unwrap(),
                    self.designation as i32,
                    self.get_full_path(),
                ),
                utility.preferences.content_output_whitelisted,
                utility,
            );
        }
    }

    pub fn get_parent_directory_from_pathbuf_as_string(pathbuf: &Path) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn content_is_episode(&self) -> bool {
        self.show_uid.is_some() && self.show_title.is_some() && self.show_season_episode.is_some()
    }

    pub fn designate_and_fill(
        &mut self,
        working_shows: &mut Vec<Show>,
        utility: Utility,
        connection: &PgConnection,
    ) {
        let mut utility = utility.clone_add_location("designate_and_fill");

        let show_season_episode_temp = self.seperate_season_episode();
        if show_season_episode_temp.is_some() {
            self.designation = Designation::Episode;
            self.show_title = Some(
                self.full_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );
            self.show_season_episode = show_season_episode_temp;
            self.show_uid = Some(Show::ensure_show_exists(
                self.show_title.clone().unwrap(),
                working_shows,
                utility.clone(),
                connection,
            ));
        } else {
            self.designation = Designation::Generic;
            self.show_title = None;
            self.show_season_episode = None;
        }

        utility.print_function_timer();
    }

    pub fn print_content(content: &[Content], utility: Utility) {
        let mut utility = utility.clone_add_location("print_content(FileManager)");

        if !utility.preferences.print_content && !utility.preferences.content_output_whitelisted {
            return;
        }

        for content in content {
            content.print(utility.clone());
        }

        utility.print_function_timer();
    }
}

fn rem_first_char(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}
