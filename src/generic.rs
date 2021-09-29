//!Datatype and associated function for handling Generic video files as well as the generic
//!information used by all other video file types
use std::io::prelude::*;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use std::fmt;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use tracing::{event, Level};

use crate::config::Preferences;
use crate::{
    designation::{convert_i32_to_designation, Designation},
    model::*,
    profile::Profile,
};

///Struct containing data that is shared by all file types
///can also refer to only a generic media file
#[derive(Clone, Debug)]
pub struct Generic {
    pub generic_uid: Option<usize>,
    pub full_path: PathBuf,
    pub designation: Designation,
    pub hash: Option<String>,
    pub fast_hash: Option<String>,
    pub profile: Option<Profile>,
}

impl fmt::Display for Generic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_full_path())
    }
}

impl Generic {
    pub fn new(raw_filepath: &Path) -> Self {
        Generic {
            full_path: raw_filepath.to_path_buf(),
            designation: Designation::Generic,
            generic_uid: None,
            hash: None,
            fast_hash: None,
            profile: Profile::from_file(raw_filepath.to_path_buf()),
        }
    }

    ///Hash the file with seahash for data integrity purposes so we
    /// know if a file has been replaced and may need to be reprocessed
    pub fn hash(&mut self) {
        let mut buffer = Box::new(vec![0; 4096]);
        let mut hasher = seahash::SeaHasher::new();
        let mut file = File::open(self.full_path.to_str().unwrap()).unwrap();
        while file.read(&mut buffer).unwrap() != 0 {
            hasher.write(&buffer);
        }
        self.hash = Some(hasher.finish().to_string());
    }

    ///Hash the first 32MB of the file with seahash so we can quickly know
    ///if a file is likely to have changed or is likely to be the same as
    ///an existing file.
    ///
    ///For example if we backup all of tlm's information and all files get
    ///renamed to something that doesn't make sense we can quickly search for
    ///files that tlm knows about to restore by calculating the fast hash and
    ///then calculating full hashes of matching hashes to save time
    pub fn fast_hash(&mut self) {
        let mut buffer = Box::new(vec![0; 4096]);
        let mut hasher = seahash::SeaHasher::new();
        let mut file = File::open(self.full_path.to_str().unwrap()).unwrap();
        for _ in 0..8192 {
            if file.read(&mut buffer).unwrap() != 0 {
                hasher.write(&buffer);
            } else {
                break;
            }
        }
        let fast_hash = seahash::hash(&buffer);
        self.fast_hash = Some(fast_hash.to_string());
    }

    ///Create a new generic from the database equivalent. This is neccesary because
    /// not all fields are stored in the database because they can be so easily recalculated
    pub fn from_generic_model(generic_model: GenericModel) -> Generic {
        let generic_uid_temp: i32 = generic_model.generic_uid;
        let full_path_temp: String = generic_model.full_path.to_owned();
        let designation_temp: i32 = generic_model.designation;

        //change to have it pull all info out of the db, it currently generates what it can from the filename
        Generic {
            full_path: PathBuf::from(&full_path_temp),
            designation: convert_i32_to_designation(designation_temp), //Designation::Generic
            generic_uid: Some(generic_uid_temp as usize),
            hash: generic_model.file_hash.to_owned(),
            fast_hash: generic_model.fast_file_hash.to_owned(),
            profile: generic_model.get_profile(),
        }
    }

    ///Returns a vector of ffmpeg arguments for later execution
    ///This has no options currently
    #[allow(dead_code)]
    fn generate_encode_string(&self) -> Vec<String> {
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
    fn generate_target_path(&self) -> String {
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

    fn get_full_path_with_suffix(&self, suffix: String) -> PathBuf {
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

    pub fn get_full_path(&self) -> String {
        return self.full_path.as_os_str().to_str().unwrap().to_string();
    }

    pub fn get_generic_uid(&self) -> usize {
        if self.generic_uid.is_some() {
            self.generic_uid.unwrap()
        } else {
            panic!("get_generic_uid was called on a generic that hasn't been inserted into the db yet or hasn't been assigned a generic_uid from the database correctly");
        }
    }

    pub fn get_filename_from_pathbuf(pathbuf: PathBuf) -> String {
        return pathbuf.file_name().unwrap().to_str().unwrap().to_string();
    }

    fn print_generic(&self) {
        event!(
            Level::DEBUG,
            "[generic_uid:'{:4}'][designation:'{}'][full_path:'{}']",
            self.get_generic_uid(),
            self.designation as i32,
            self.get_full_path(),
        );
    }

    pub fn print_generics(generics: &[Generic], preferences: &Preferences) {
        if !preferences.print_generic && !preferences.generic_output_whitelisted {
            return;
        }

        for generic in generics {
            generic.print_generic();
        }
    }
}
