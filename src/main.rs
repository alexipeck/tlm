use regex::Regex;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use std::process::Command; //borrow::Cow, thread::current,
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use twox_hash::xxh3;
use walkdir::WalkDir;

static UID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
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

fn get_next_unreserved(queue: Queue) -> Option<usize> {
    for content in queue.priority_queue {
        if content.reserved_by == None {
            return Some(content.uid);
        }
    }

    for content in queue.main_queue {
        if content.reserved_by == None {
            return Some(content.uid);
        }
    }

    return None;
}

fn exec(command: &Vec<&str>) -> String {
    let buffer;
    if !cfg!(target_os = "windows") {
        //linux & friends
        buffer = Command::new("ffmpeg")
            .args(command)
            .output()
            .expect("failed to execute process");
    } else {
        //windows
        buffer = Command::new("ffmpeg")
            .args(command)
            .output()
            .expect("failed to execute process");
    }
    String::from_utf8_lossy(&buffer.stdout).to_string()
}

#[derive(Clone, Copy)]
enum Designation {
    Generic,
    Episode,
    Movie,
}

//generic content container, focus on video
#[derive(Clone)]
struct Content {
    uid: usize,
    full_path: String,
    designation: Designation,
    parent_directory: String,
    filename: String,
    filename_woe: String,
    reserved_by: Option<String>,

    hash: Option<u64>,
    //versions: Vec<FileVersion>,
    //metadata_dump
    show_title: Option<String>,
    show_season_episode: Option<(String, String)>,
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

        //prepare full path
        let full_path = format!("{}{}", parent_directory, filename);

        let designation: Designation;
        if episode {
            designation = Designation::Episode;
        } else {
            designation = Designation::Generic;
        }
        Content {
            full_path: full_path,
            designation: designation,
            uid: UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            parent_directory: parent_directory,
            filename: filename,
            filename_woe: filename_woe,
            reserved_by: None,
            hash: None,

            show_title: None,
            show_season_episode: None,
        }
    }
}

//doesn't handle errors correctly
fn prioritise_content_by_title(queue: &mut Queue, filenames: Vec<String>) {
    for _ in 0..filenames.len() {
        let mut index: usize = 0;
        let mut found = false;
        for content in &queue.main_queue {
            for filename in &filenames {
                if content.filename == *filename {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
            index += 1;
        }
        if found {
            queue.priority_queue.push(queue.main_queue.remove(index));
        }
    }
}

fn prioritise_content_by_uid(queue: &mut Queue, uids: Vec<usize>) {
    for _ in 0..uids.len() {
        let mut index: usize = 0;
        let mut found = false;
        for content in &queue.main_queue {
            for uid in &uids {
                if content.uid == *uid {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
            index += 1;
        }
        if found {
            queue.priority_queue.push(queue.main_queue.remove(index));
        }
    }
}

struct Season {
    number: u8,
    episodes: Vec<Content>,
}

struct Show {
    title: String,
    seasons: Vec<Season>,
}

fn rem_first_char(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}

//requires raw string expression
fn re_strip(input: &String, expression: &str) -> Option<String> {
    let output = Regex::new(expression).unwrap().find(input);
    match output {
        None => return None,
        Some(val) => return Some(String::from(rem_first_char(val.as_str()))),
    }
}

struct Queue {
    priority_queue: Vec<Content>,
    main_queue: Vec<Content>,
}

fn assign_uid() {}

//Return true in string contains any substring from Vector
fn str_contains_strs(input_str: &str, substrings: &Vec<&str>) -> bool {
    for substring in substrings {
        if String::from(input_str).contains(substring) {
            return true;
        }
    }
    false
}

fn import_files(
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
                    file_paths.push(entry.into_path());
                }
            }
        }
    }
}

fn main() {
    //Content UID
    let mut next_available_uid: usize = 0;

    //Queue
    let mut queue = Queue {
        priority_queue: Vec::new(),
        main_queue: Vec::new(),
    };

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_root_directories.push(String::from("/home/ryan/test_files")); //manual entry
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_root_directories.push(String::from(r"C:\Users\Alexi Peck\Desktop\tlm\test_files\"));
        //manual entry
    }

    //allowed video extensions
    let allowed_extensions = vec!["mp4", "mkv", "webm", "MP4"];

    //ignored directories
    //currently works on both linux and windows
    let ignored_paths = vec![".recycle_bin"];

    let mut raw_filepaths = Vec::new();

    //Load all video files under tracked directories exluding all ignored paths
    import_files(
        &mut raw_filepaths,
        &tracked_root_directories,
        &allowed_extensions,
        &ignored_paths,
    );

    //sort out filepaths into series and seasons
    let mut shows: Vec<Show> = Vec::new();

    //loop through all paths
    for raw_filepath in raw_filepaths {
        let mut content = Content::new(&raw_filepath);

        //prepare title
        let mut show_title = String::new();
        for section in String::from(
            raw_filepath
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_string_lossy(),
        )
        .split('/')
        .rev()
        {
            show_title = String::from(section);
            break;
        }

        //dumping prepared values into Content struct based on Designation
        match content.designation {
            Designation::Episode => {
                let mut episode = false;
                let season_episode = seperate_season_episode(&content.filename, &mut episode);

                content.show_title = Some(show_title);
                content.show_season_episode = season_episode;

                //index of the current show in the shows vector
                let mut current_show = 0;

                //determine whether the show exists in the shows vector, if it does, it saves the index
                let mut exists = false;
                for (i, show) in shows.iter().enumerate() {
                    if show.title == *(content.show_title.as_ref().unwrap()) {
                        exists = true;
                        current_show = i;
                        break;
                    }
                }

                //if the show doesn't exist in the vector, it creates it, and saves the index
                if !exists {
                    let show = Show {
                        title: content.show_title.as_ref().unwrap().clone(),
                        seasons: Vec::new(),
                    };
                    shows.push(show);
                    current_show = shows.len() - 1;
                }

                //determines whether the season exists in the seasons vector of the current show, if it does, it saves the index
                exists = false;
                let mut current_season: usize = 0; //content.show_season_episode.0.parse::<usize>().unwrap()
                for (i, season) in shows[current_show].seasons.iter().enumerate() {
                    if season.number
                        == content
                            .show_season_episode
                            .as_ref()
                            .unwrap()
                            .0
                            .parse::<u8>()
                            .unwrap()
                    {
                        exists = true;
                        current_season = i;
                        break;
                    }
                }

                //if the season doesn't exist in the current show's seasons vector, it creates it
                if !exists {
                    let season = Season {
                        number: content
                            .show_season_episode
                            .as_ref()
                            .unwrap()
                            .0
                            .parse::<u8>()
                            .unwrap(),
                        episodes: Vec::new(),
                    };

                    shows[current_show].seasons.push(season);

                    current_season = shows[current_show].seasons.len() - 1;
                }
                //push episode to current season
                shows[current_show].seasons[current_season]
                    .episodes
                    .push(content.clone());
            }
            /*Designation::Movie => (

            ),*/
            _ => {}
        }
        queue.main_queue.push(content);
    }
    let mut filenames: Vec<String> = Vec::new();
    //filenames.push(String::from(r"Weeds - S08E10 - Threshold Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E11 - God Willing and the Creek Don't Rise Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E12-13 - It's Time Bluray-1080p.mkv"));

    let mut uids: Vec<usize> = Vec::new();
    //uids.push(10);
    //uids.push(22);
    //uids.push(35);

    prioritise_content_by_title(&mut queue, filenames.clone());

    prioritise_content_by_uid(&mut queue, uids.clone());

    for content in &queue.priority_queue {
        println!("{}{}", content.parent_directory, content.filename);
    }

    for content in &queue.main_queue {
        println!("{}{}", content.parent_directory, content.filename);
    }

    for content in queue.main_queue {
        let source = format!("{}{}", content.parent_directory, content.filename);
        let target = format!(
            "{}{}_h265.mp4",
            content.parent_directory, content.filename_woe
        );
        println!(
            "Starting encode of {}\nEncoding to {}_h265.mp4",
            content.filename, content.filename_woe
        );
        //async
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
            &target,
        ];

        //let encode_string: String = format!("ffmpeg -i \"{}\" -c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k \"{}\"", source, target);
        //println!("Source: {}\nTarget: {}\nEncode string: {}", source, target, encode_string);
        let output = exec(&encode_string);
        println!("{}", output);
    }

    if false {
        for show in &shows {
            //println!("{}", show.title);
            for season in &show.seasons {
                //println!("{}", season.number);
                for episode in &season.episodes {
                    println!("{}{}", episode.parent_directory, episode.filename,);
                }
            }
        }
    }

    //add to db by filename, allowing the same file to be retargeted in another directory, without losing track of all the data associated with the episode

    //unify generic and episode naming (bring together)

    //println!("Converting file to h265, no estimated time currently");
    //exec("ffmpeg -i W:/tlm/test_files/tf1.mp4 -c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k W:/tlm/test_files/tf1_h265.mp4");
}
