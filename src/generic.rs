use std::{collections::HashSet, fs, path::PathBuf};

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

/// this will obviously mean memory overhead. In future I think
/// we should split this into 3 types that would however mean
/// that the manager would require seperate vectors but I consider
/// that a non issue
#[derive(Clone, Debug, Queryable)]
pub struct Generic {
    pub generic_uid: Option<usize>,
    pub full_path: PathBuf,
    pub designation: Designation,
    pub hash: Option<String>,
    pub profile: Option<Profile>,    
}

impl Generic {
    //needs to be able to be created from a pathbuf or pulled from the database
    pub fn new(
        raw_filepath: &PathBuf,
        working_shows: &mut Vec<Show>,
        utility: Utility,
        connection: &PgConnection,
    ) -> Self {
        let mut utility = utility.clone_add_location("new(Generic)");

        let mut generic = Generic {
            full_path: raw_filepath.to_path_buf(),
            designation: Designation::Generic,
            generic_uid: None,
            hash: None,
            profile: Some(Profile::new(0, 0, 0, 0)), //asdf;
        };
        generic.designate_and_fill(working_shows, utility.clone(), connection);
        utility.print_function_timer();
        return generic;
    }

    ///Hash the file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        let hash = seahash::hash(&fs::read(self.full_path.to_str().unwrap()).unwrap());
        self.hash = Some(hash.to_string());
    }

    ///Create a new content from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_generic_model(
        generic_model: GenericModel,
        working_shows: &mut Vec<Show>,
        utility: Utility,
        connection: &PgConnection,
    ) -> Generic {
        let mut utility = utility.clone_add_location("from_row(Generic)");

        let generic_uid_temp: i32 = generic_model.id;
        let full_path_temp: String = generic_model.full_path;
        let designation_temp: i32 = generic_model.designation;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let mut generic = Generic {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            generic_uid: Some(generic_uid_temp as usize),
            hash: generic_model.file_hash,
            profile: Some(Profile::new(0, 0, 0, 0)), //this is fine for now as profile isn't in the database//asdf;
        };

        generic.designate_and_fill(working_shows, utility.clone(), connection);
        utility.print_function_timer();

        return generic;
    }

    pub fn get_all_filenames_as_hashset_from_generic(
        generics: &Vec<Generic>,
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        let mut hashset = HashSet::new();
        for generic in generics {
            hashset.insert(generic.full_path.clone());
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

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_generic_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_content_uid(Show)");

        if self.generic_uid.is_some() {
            return self.generic_uid.unwrap();
        } else {
            print(
                Verbosity::CRITICAL,
                From::Generic,
                String::from("get_generic_uid was called on a generic that hasn't been inserted into the db yet or hasn't been assigned a generic_uid from the database correctly"),
                false,
                utility,
            );
            panic!();
        }
    }

    pub fn print_generic(content: &Vec<Generic>, utility: Utility) {
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
