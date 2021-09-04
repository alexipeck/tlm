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
    utility::Utility,
};

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
    pub fn new(raw_filepath: &Path, utility: Utility) -> Self {
        let mut utility = utility.clone_add_location("new(Generic)");

        let generic = Generic {
            full_path: raw_filepath.to_path_buf(),
            designation: Designation::Generic,
            generic_uid: None,
            hash: None,
            profile: Profile::from_file(raw_filepath.to_path_buf()),
        };

        utility.print_function_timer();
        generic
    }

    ///Hash the file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        let hash = seahash::hash(&fs::read(self.full_path.to_str().unwrap()).unwrap());
        self.hash = Some(hash.to_string());
    }

    ///Create a new generic from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_generic_model(generic_model: GenericModel, utility: Utility) -> Generic {
        let mut utility = utility.clone_add_location("from_row(Generic)");

        let generic_uid_temp: i32 = generic_model.generic_uid;
        let full_path_temp: String = generic_model.full_path.to_owned();
        let designation_temp: i32 = generic_model.designation;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        let generic = Generic {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            generic_uid: Some(generic_uid_temp as usize),
            hash: generic_model.file_hash.to_owned(),
            profile: generic_model.get_profile(),
        };

        utility.print_function_timer();

        generic
    }

    pub fn get_all_filenames_as_hashset_from_generics(
        generics: &[Generic],
        utility: Utility,
    ) -> HashSet<PathBuf> {
        let mut utility = utility.clone_add_location("get_all_filenames_as_hashset");
        let mut hashset = HashSet::new();
        for generic in generics {
            hashset.insert(generic.full_path.clone());
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

    /* pub fn create_job(&mut self) -> Job {
        return Job::new(self.full_path.clone(), self.generate_encode_string());
    } */

    ///Appends a fixed string to differentiate rendered files from original before overwrite
    /// I doubt this will stay as I think a temp directory would be more appropriate.
    /// This function returns that as a PathBuf
    pub fn generate_encode_path_from_pathbuf(pathbuf: PathBuf) -> PathBuf {
        Generic::get_full_path_with_suffix_from_pathbuf(pathbuf, "_encodeH4U8".to_string())
    }

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_generic_uid(&self, utility: Utility) -> usize {
        let utility = utility.clone_add_location("get_generic_uid(Show)");

        if self.generic_uid.is_some() {
            self.generic_uid.unwrap()
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

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    pub fn print_generic(&self, utility: Utility) {
        let utility = utility.clone_add_location("print_generic(Generic)");
        print(
            Verbosity::DEBUG,
            From::Generic,
            format!(
                "[generic_uid:'{:4}'][designation:'{}'][full_path:'{}']",
                self.get_generic_uid(utility.clone()),
                self.designation as i32,
                self.get_full_path(),
            ),
            utility.preferences.generic_output_whitelisted,
            utility,
        );
    }

    pub fn print_generics(generics: &[Generic], utility: Utility) {
        let mut utility = utility.clone_add_location("print_generics(Generic)");

        if !utility.preferences.print_generic && !utility.preferences.generic_output_whitelisted {
            return;
        }

        for generic in generics {
            generic.print_generic(utility.clone());
        }

        utility.print_function_timer();
    }
}
