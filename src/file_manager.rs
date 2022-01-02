//!A struct for managing all types of media that are stored in ram as well as
//!Functionality to import files. This is mostly used in the scheduler
use crate::{
    config::ServerConfig,
    database::*,
    designation::Designation,
    encode::{Encode, EncodeProfile},
    generic::{FileVersion, Generic},
    get_show_title_from_pathbuf,
    model::{NewEpisode, NewFileVersion, NewGeneric},
    get_extension, get_file_stem, to_string,
    show::{Episode, Show}, ensure_path_exists,
};
extern crate derivative;
use derivative::Derivative;
use diesel::pg::PgConnection;
use jwalk::WalkDir;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::Path};
use std::{collections::HashSet, fmt, path::PathBuf};
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};
use tracing::{error, info, trace, warn};
///Struct to hold all root directories containing media
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct TrackedDirectories {
    root_directories: HashSet<PathBuf>,
    cache_directory: Option<PathBuf>,
    //This needs to be accessible by the workers
    global_temp_directory: Option<PathBuf>,
}

impl TrackedDirectories {
    pub fn new_empty() -> Self {
        Self {
            root_directories: HashSet::new(),
            cache_directory: None,
            global_temp_directory: None,
        }
    }

    pub fn default() -> Self {
        let mut tracked_directories = Self::new_empty();
        tracked_directories.assign_cache_directory(None);
        tracked_directories
    }

    pub fn has_cache_directory(&self) -> bool {
        self.cache_directory.is_some()
    }

    pub fn has_global_temp_directory(&self) -> bool {
        self.global_temp_directory.is_some()
    }

    pub fn add_root_directory(&mut self, tracked_directory: PathBuf) {
        self.root_directories.insert(tracked_directory);
    }

    //Destructive
    pub fn assign_cache_directory(&mut self, directory: Option<&Path>) {
        let cache_directory;
        if let Some(directory) = directory {
            //should be able to determine whether there has already been a /tlm/ folder created in the passed cache directory
            if to_string(&get_file_stem(directory)) != "tlm" {
                cache_directory = directory.join("tlm");
            } else {
                cache_directory = directory.to_path_buf();
            }
        } else {
            cache_directory = env::temp_dir().join("tlm");
        }
        ensure_path_exists(&cache_directory);
        self.cache_directory = Some(cache_directory);
    }

    //Destructive
    pub fn assign_global_temp_directory(&mut self, global_temp_directory: &Path) {
        self.global_temp_directory = Some(PathBuf::from(global_temp_directory));
    }

    pub fn get_global_temp_directory(&self) -> &PathBuf {
        //is probably guaranteed
        match self.global_temp_directory.as_ref() {
            Some(temp_directory) => temp_directory,
            None => {
                error!("For any local transcoding, just a local directory is fine, but this needs to be accessible for any remote worker.");
                panic!();
            }
        }
    }

    pub fn get_root_directories(&self) -> &HashSet<PathBuf> {
        &self.root_directories
    }

    pub fn get_cache_directory(&self) -> &PathBuf {
        if self.cache_directory.is_some() {
            return self.cache_directory.as_ref().unwrap();
        } else {
            error!("This should not be called before a cache directory has been set.");
            panic!();
        }
    }
}

///This enum represents the list of reasons that a file was not imported.
///This is used to create a log of files that weren't imported so
///that the user can determine the reason that some of their media
///was not imported if they expected it to be
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Hash)]
enum Reason {
    PathContainsIgnoredPath,
    ExtensionMissing,
    ExtensionDisallowed,
}

struct PathBufReason {
    pathbuf: PathBuf,
    reason: Reason,
}

impl PartialEq for PathBufReason {
    fn eq(&self, other: &Self) -> bool {
        self.pathbuf == other.pathbuf
    }
}

impl Eq for PathBufReason {}

impl Hash for PathBufReason {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pathbuf.hash(state);
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatted: &str = match self {
            Self::PathContainsIgnoredPath => "PathContainsIgnoredPath",
            Self::ExtensionMissing => "ExtensionMissing",
            Self::ExtensionDisallowed => "ExtensionDisallowed",
        };

        write!(f, "{}", formatted)
    }
}

///Contains all media data that is stored in ram as well as
///a list of rejected files
pub struct FileManager {
    pub config: Arc<RwLock<ServerConfig>>,
    pub generic_files: Vec<Generic>,
    pub shows: Vec<Show>,
    pub existing_files_hashset: HashSet<PathBuf>,
    pub new_files_queue: Vec<PathBuf>,
    rejected_files: HashSet<PathBufReason>,
}

impl FileManager {
    pub fn new(config: Arc<RwLock<ServerConfig>>) -> Self {
        let mut file_manager = Self {
            config,
            shows: get_all_shows(),
            generic_files: Vec::new(),
            existing_files_hashset: HashSet::new(),
            new_files_queue: Vec::new(),
            rejected_files: HashSet::new(),
        };

        //add generic_files and generics from their respective episodes to the existing_files_hashset
        file_manager.generic_files = get_all_generics();
        let mut collected_file_versions: HashMap<i32, Vec<FileVersion>> = HashMap::new();
        {
            for file_version in get_all_file_versions() {
                match collected_file_versions.get_mut(&file_version.generic_uid) {
                    Some(value) => {
                        value.push(file_version.clone());
                    }
                    None => {
                        collected_file_versions
                            .insert(file_version.generic_uid, vec![file_version.clone()]);
                    }
                }
            }
        }

        //Make sure the master file is first in the list if it isn't already, if none exist set it as the first by default with a warning
        for (generic_id, file_versions) in collected_file_versions.iter_mut() {
            if !file_versions[0].master_file {
                let mut master_index_found = false;
                for (i, t) in file_versions.iter().enumerate() {
                    if t.master_file {
                        file_versions.swap(i, 0);
                        master_index_found = true;
                        break;
                    }
                }

                //Default to first file in list if none is found but warn the user about it
                if !master_index_found {
                    warn!("No master file found for generic with id: {}. Defaulting to first file in list with id: {}", generic_id, file_versions[0].id);
                    file_versions[0].master_file = true;
                }
            }
        }

        for generic in file_manager.generic_files.iter_mut() {
            if generic.generic_uid.is_none() {
                error!("This generic doesn't have a generic_uid, this shouldn't happen.");
                panic!();
            }
            let generic_uid = generic.generic_uid.unwrap();

            if let Some(file_versions) = collected_file_versions.get(&generic_uid) {
                generic.file_versions = file_versions.to_owned();
            }
        }

        file_manager.add_existing_files_to_hashset();
        file_manager.add_show_episode_file_versions_to_hashset();
        file_manager
    }

    //Returns true if successful
    pub fn insert_file_version(&mut self, file_version: &FileVersion) -> bool {
        for generic in self.generic_files.iter_mut() {
            if generic.generic_uid.is_none() {
                error!("An action was taken on a generic that doesn't have a generic_uid, this shouldn't happen.");
                panic!();
            }
            if generic.generic_uid.unwrap() == file_version.generic_uid {
                generic.file_versions.push(file_version.clone());
                return true;
            }
        }

        for show in self.shows.iter_mut() {
            if show.insert_file_version(file_version) {
                return true;
            }
        }
        false
    }

    pub fn get_encode_from_generic_uid(
        &self,
        generic_uid: i32,
        file_version_id: i32,
        encode_profile: &EncodeProfile,
        server_config: Arc<RwLock<ServerConfig>>,
    ) -> Option<Encode> {
        for generic in &self.generic_files {
            if generic.get_generic_uid() == generic_uid {
                if let Some(file_version) = generic.get_file_version_by_id(file_version_id) {
                    return Some(Encode::new(&file_version, encode_profile, &server_config));
                }
            }
        }
        for show in &self.shows {
            if let Some(generic) = show.get_generic_from_uid(generic_uid) {
                if let Some(file_version) = generic.get_file_version_by_id(file_version_id) {
                    return Some(Encode::new(&file_version, encode_profile, &server_config));
                }
            }
        }
        None
    }

    ///Takes all loaded episodes and adds their all their file_versions to the hashset
    ///of existing files to ensure that files don't get imported twice
    fn add_show_episode_file_versions_to_hashset(&mut self) {
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    let generic = &episode.generic;
                    for file_version in &generic.file_versions {
                        self.existing_files_hashset
                            .insert(file_version.full_path.clone());
                    }
                }
            }
        }
    }

    ///Adds all generics that exist in the file manager to the hashset to ensure that
    ///files can't be imported twice
    fn add_existing_files_to_hashset(&mut self) {
        for generic in &self.generic_files {
            for path in generic.get_all_full_paths() {
                self.existing_files_hashset.insert(path);
            }
        }
    }

    pub fn print_number_of_generics(&self) {
        info!(
            "Number of generics loaded in memory: {}",
            self.generic_files.len()
        );
    }

    pub fn print_number_of_shows(&self) {
        info!("Number of shows loaded in memory: {}", self.shows.len());
    }

    pub fn print_number_of_episodes(&self) {
        let mut episode_counter = 0;
        for show in &self.shows {
            for season in &show.seasons {
                episode_counter += season.episodes.len();
            }
        }

        info!("Number of episodes loaded in memory: {}", episode_counter);
    }

    ///Processes all files in the new files queue and converts them to
    ///episodes and generics based on pattern matching. So far only accepts
    ///The filename pattern SxxExx where Sxx is the season number and Exx
    ///is the episode number
    pub fn process_new_files(&mut self) {
        let connection = establish_connection();
        let mut new_episodes = Vec::new();
        let mut new_generics = Vec::new();
        let mut new_file_versions = Vec::new();

        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"S[0-9]*E[0-9\-]*").unwrap();
        }

        let mut generics: Vec<Generic> = Vec::new();
        //Indented so temp_generics_and_paths drops out of scope earlier
        {
            //Create Generic and NewGeneric that will be added to the database in a batch
            let mut temp_generics_and_paths: Vec<(Generic, String)> = self
                .new_files_queue
                .par_iter()
                .map(|current| {
                    let mut generic = Generic::default();
                    let master_file_path = to_string(current);
                    //TODO: Why yes this is slower, no I don't care about 100ms right now
                    match REGEX.find(&master_file_path) {
                        None => {}
                        Some(_) => generic.designation = Designation::Episode,
                    }

                    (generic, master_file_path)
                })
                .collect();
            self.new_files_queue.clear();
            for (generic, _) in &temp_generics_and_paths {
                new_generics.push(NewGeneric::new(generic.designation as i32));
            }

            //Insert the generic and then update the uid's for the full Generic structure
            let generic_models = create_generics(&connection, new_generics);
            for i in 0..generic_models.len() {
                temp_generics_and_paths[i].0.generic_uid = Some(generic_models[i].generic_uid);
            }

            for (generic, full_path) in temp_generics_and_paths {
                new_file_versions.push(NewFileVersion::new(
                    generic.generic_uid.unwrap(),
                    full_path,
                    true,
                ));
                generics.push(generic);
            }
        }

        let file_versions = create_file_versions(&connection, new_file_versions);
        for (i, generic) in generics.iter_mut().enumerate() {
            generic
                .file_versions
                .push(FileVersion::from_file_version_model(
                    file_versions[i].clone(),
                ));
            trace!("Processed {}", generic);
        }

        //Build all the NewEpisodes so we can do a batch insert that is faster than doing one at a time in a loop
        for generic in generics.iter_mut() {
            let episode_string: String;
            match REGEX.find(&generic.file_versions[0].get_filename()) {
                None => continue,
                Some(val) => {
                    //Removes first character
                    let mut chars = val.as_str().chars();
                    chars.next();

                    episode_string = String::from(chars.as_str());
                }
            }

            let mut season_episode_iter = episode_string.split('E');
            let season_temp = season_episode_iter.next().unwrap().parse::<i32>().unwrap();
            let mut episodes: Vec<i32> = Vec::new();
            for episode in season_episode_iter.next().unwrap().split('-') {
                episodes.push(episode.parse::<i32>().unwrap());
            }

            generic.designation = Designation::Episode;
            let show_title = get_show_title_from_pathbuf(&generic.file_versions[0].full_path);

            let show_uid = self.ensure_show_exists(show_title.clone(), &connection);
            let season_number = season_temp;
            let episode_number = episodes[0];

            let new_episode = NewEpisode::new(
                generic.get_generic_uid(),
                show_uid,
                "".to_string(), //episode_title
                season_number,
                episode_number,
            );
            new_episodes.push(new_episode);
        }

        let mut temp_generics_only_episodes: Vec<Generic> = Vec::new();
        let mut temp_generics_only_generics: Vec<Generic> = Vec::new();
        for generic in &generics {
            match generic.designation {
                Designation::Generic => temp_generics_only_generics.push(generic.clone()),
                Designation::Episode => temp_generics_only_episodes.push(generic.clone()),
                _ => {}
            }
        }

        self.generic_files.append(&mut temp_generics_only_generics);

        let mut episodes: Vec<Episode> = Vec::new();
        for episode_model in create_episodes(&connection, new_episodes) {
            for generic in &temp_generics_only_episodes {
                if generic.get_generic_uid() == episode_model.generic_uid {
                    let episode = Episode::new(
                        generic.clone(),
                        episode_model.show_uid,
                        "".to_string(),
                        episode_model.season_number,
                        vec![episode_model.episode_number],
                    ); //temporary first episode_number
                    episodes.push(episode);
                    break;
                }
            }
        }

        self.insert_episodes(episodes);
    }

    pub fn generate_profiles(&mut self) {
        let connection = &establish_connection();
        for generic in self.generic_files.iter_mut() {
            generic.generate_file_version_profiles_if_none(connection);
        }

        for show in self.shows.iter_mut() {
            for season in show.seasons.iter_mut() {
                for episode in season.episodes.iter_mut() {
                    episode
                        .generic
                        .generate_file_version_profiles_if_none(connection);
                }
            }
        }
    }

    ///returns none when a file is rejected because is accepted, or already exists in the existing_files_hashset
    fn accept_or_reject_file(&mut self, full_path: PathBuf, store_reasons: bool) {
        let mut reason = None;
        //rejects if the path contains any element of an ignored path
        for ignored_path in &self.config.read().unwrap().ignored_paths_regex {
            if ignored_path
                .is_match(&to_string(&full_path))
                .unwrap()
            {
                reason = Some(Reason::PathContainsIgnoredPath);
            }
        }

        //rejects if the path doesn't have an extension
        if reason.is_none() {
            if full_path.extension().is_none() {
                reason = Some(Reason::ExtensionMissing);
            } else {
                //rejects if the file doesn't have an allowed extension
                if !self
                    .config
                    .read()
                    .unwrap()
                    .allowed_extensions
                    .contains(&to_string(&get_extension(&full_path)).to_lowercase())
                {
                    reason = Some(Reason::ExtensionDisallowed);
                }
            }
        }

        if let Some(reason) = reason {
            if store_reasons {
                trace!("Rejected {} for {}", to_string(&full_path), reason);
                self.rejected_files.insert(PathBufReason {
                    pathbuf: full_path,
                    reason,
                });
            }
        } else if self.existing_files_hashset.insert(full_path.clone()) {
            self.new_files_queue.push(full_path);
        }
    }

    ///Import all files in the list of tracked root directories
    ///into a queue for later processing. Uses a Hash set to
    ///guarantee no duplicates in O(1) time
    pub fn import_files(&mut self) {
        //import all files in tracked root directories
        let root_directories = &self
            .config
            .read()
            .unwrap()
            .clone()
            .tracked_directories
            .root_directories;
        for directory in root_directories {
            //If we do thi first we can max out IO without waiting
            //for accept_or_reject files. Will increase memory overhead obviously
            for entry in WalkDir::new(directory) {
                if entry.as_ref().unwrap().path().is_file() {
                    self.accept_or_reject_file(entry.unwrap().path(), true);
                }
            }
        }
    }

    pub fn generate_encodes_for_all(&self, encode_profile: &EncodeProfile) -> Vec<Encode> {
        let mut encodes: Vec<Encode> = Vec::new();
        for generic in &self.generic_files {
            for file_version in &generic.file_versions {
                encodes.push(Encode::new(file_version, encode_profile, &self.config));
            }
        }
        for show in &self.shows {
            for season in &show.seasons {
                for episode in &season.episodes {
                    for file_version in &episode.generic.file_versions {
                        encodes.push(Encode::new(file_version, encode_profile, &self.config));
                    }
                }
            }
        }
        encodes
    }

    ///Insert a vector of episodes into an existing show
    pub fn insert_episodes(&mut self, episodes: Vec<Episode>) {
        //find the associated show
        //insert episode into that show
        for episode in episodes {
            let show_uid = episode.show_uid;
            for show in &mut self.shows {
                if show.show_uid == show_uid {
                    show.insert_episode(episode);
                    break;
                }
            }
        }
    }

    ///Check if a show exists in ram
    fn show_exists(&self, show_title: String) -> Option<i32> {
        for show in &self.shows {
            if show.show_title == show_title {
                return Some(show.show_uid);
            }
        }
        None
    }

    ///Make sure a show exists by checking for it in ram and inserting it into
    ///the database if it doesn't exist yet
    fn ensure_show_exists(&mut self, show_title: String, connection: &PgConnection) -> i32 {
        let show_uid = self.show_exists(show_title.clone());
        match show_uid {
            Some(uid) => uid,
            None => {
                let show_model = create_show(connection, show_title.clone());

                let show_uid = show_model.show_uid;
                let new_show = Show {
                    show_uid,
                    show_title,
                    seasons: Vec::new(),
                };
                self.shows.push(new_show);

                show_uid
            }
        }
    }

    pub fn print_rejected_files(&self) {
        for file in &self.rejected_files {
            info!(
                "Path: '{}' disallowed because {}",
                to_string(&file.pathbuf),
                file.reason
            );
        }
    }
}
