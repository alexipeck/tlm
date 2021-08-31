use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    database::get_all_content,
    designation::{convert_i32_to_designation, Designation},
    model::*,
    print::{print, From, Verbosity},
    profile::Profile,
    tv::Show,
    utility::Utility,
};
use regex::Regex;

///Generic contains fields for generic, episode, and movie data
/// this will obviously mean memory overhead. In future I think
/// we should split this into 3 types that would however mean
/// that the manager would require seperate vectors but I consider
/// that a non issue
#[derive(Clone, Debug, Queryable)]
pub struct Generic {
    pub content_uid: Option<usize>,
    pub full_path: PathBuf,
    pub designation: Designation,
    pub hash: Option<String>,
    pub profile: Option<Profile>,
}

impl Generic {
    //needs to be able to be created from a pathbuf or pulled from the database
    pub fn new(raw_filepath: &PathBuf, utility: Utility) -> Self {
        let mut utility = utility.clone_add_location("new(Generic)");

        let mut generic = Self {
            full_path: raw_filepath.to_path_buf(),
            designation: Designation::Generic,
            content_uid: None,
            hash: None,
            profile: Some(Profile::new(0, 0, 0, 0)), //asdf;
        };
        utility.print_function_timer();
        return generic;
    }

    pub fn get_generic_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_generic_uid(Generic)");

        if self.content_uid.is_some() {
            return self.content_uid.unwrap();
        } else {
            print(
                Verbosity::CRITICAL,
                From::Generic,
                String::from("get_generic_uid was called on a content that hasn't been inserted into the db yet or hasn't been assigned a content_uid from the database correctly"),
                false,
                utility,
            );
            panic!();
        }
    }

    ///Hash the content file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        let hash = seahash::hash(&fs::read(self.full_path.to_str().unwrap()).unwrap());
        self.hash = Some(hash.to_string());
    }

    ///Create a new content from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_content_model(content_model: ContentModel, utility: Utility) -> Generic {
        let mut utility = utility.clone_add_location("from_content_model(Generic)");

        let content_uid_temp: i32 = content_model.id;
        let full_path_temp: String = content_model.full_path;
        let designation_temp: i32 = content_model.designation;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let mut content = Generic {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            content_uid: Some(content_uid_temp as usize),
            hash: content_model.file_hash,
            profile: Some(Profile::new(0, 0, 0, 0)), //this is fine for now as profile isn't in the database
        };

        utility.print_function_timer();

        return content;
    }

    pub fn get_all_filenames_as_hashset_from_content(
        contents: Vec<Generic>,
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        let mut hashset = HashSet::new();
        for content in contents {
            hashset.insert(content.full_path);
        }

        utility.print_function_timer();
        return hashset;
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
        return Generic::get_full_path_with_suffix_from_pathbuf(pathbuf, "_encodeH4U8".to_string());
    }

    pub fn seperate_season_episode(&mut self) -> Option<(usize, Vec<usize>)> {
        let episode_string: String;

        //Check if the regex caught a valid episode format
        match re_strip(&self.get_filename(), r"S[0-9]*E[0-9\-]*") {
            None => {
                return None;
            }
            Some(temp_string) => {
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

    pub fn print(&self, utility: Utility) {
        let utility = utility.clone_add_location("print(Generic)");

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
            utility.clone(),
        );
    }

    pub fn get_parent_directory_from_pathbuf_as_string(pathbuf: &PathBuf) -> String {
        return pathbuf.parent().unwrap().to_string_lossy().to_string();
    }

    pub fn print_content(content: &Vec<Generic>, utility: Utility) {
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
