//extern crate yaml_rust;
use regex::Regex;
use std::{process::Command}; //borrow::Cow, thread::current,
use walkdir::WalkDir;
use twox_hash::xxh3;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
}

fn exec(command: &str) -> String {
    let buffer;
    if !cfg!(target_os = "windows") {
        //linux & friends
        buffer = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("failed to execute process");
    } else {
        //windows
        buffer = Command::new("cmd")
            .arg(command)
            .output()
            .expect("failed to execute process");
    }
    String::from_utf8_lossy(&buffer.stdout).to_string()
}

//generic content container, focus on video
struct Content {
    parent_directory: String,
    original_filename: String,
    show_title: String,
    show_season_episode: (String, String),
    reserved_status_by: (bool, String),
    hash: Option<u64>,
    //versions: Vec<FileVersion>,
    //metadata_dump
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
fn re_strip(input: &String, expression: &str) -> String {
    return String::from(rem_first_char(
        Regex::new(expression).unwrap().find(input).unwrap().as_str(),
    ));
}

struct Queue {
    priority_queue: Vec<Content>,
    main_queue: Vec<Content>, 
}

fn main() {
    //Queue
    let mut queue = Queue {
        priority_queue: Vec::new(),
        main_queue: Vec::new(),
    };

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
    let allowed_extensions = vec!["mp4","mkv","MP4"];

    //import all files in tracked root directories
    let mut raw_filepaths = Vec::new();
    for tracked_root_directory in tracked_root_directories {
        for entry in WalkDir::new(tracked_root_directory)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if allowed_extensions.contains(&entry.path().extension().unwrap().to_str().unwrap()) {
                    raw_filepaths.push(entry.into_path());
                }
            }
        }
    }

    //sort out filepaths into series and seasons
    let mut shows: Vec<Show> = Vec::new();
    
    //loop through all paths
    for raw_filepath in raw_filepaths {
        //prepare original_filename
        let original_filename = String::from(raw_filepath.file_name().unwrap().to_string_lossy());
        
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
        
        //prepare season and episode number
        let season_episode_temp = re_strip(&original_filename, r"S[0-9]*E[0-9\-]*");
        let mut se_iter = season_episode_temp.split('E');
        let season_episode: (String, String) = (se_iter.next().unwrap().to_string(), se_iter.next().unwrap().to_string());

        //dumping prepared values into Content struct
        let content = Content {
            parent_directory: String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/"),
            original_filename: original_filename,
            show_title: show_title,
            show_season_episode: season_episode,
            reserved_status_by: (false, String::new()),
            hash: None,
            //hash: Some(hash_file(raw_filepath)),
        };

        //index of the current show in the shows vector
        let mut current_show = 0;

        //determine whether the show exists in the shows vector, if it does, it saves the index
        let mut exists = false;
        for (i, show) in shows.iter().enumerate() {
            if show.title == content.show_title {
                exists = true;
                current_show = i;
                break;
            }
        }

        //if the show doesn't exist in the vector, it creates it, and saves the index
        if !exists {
            let show = Show {
                title: content.show_title.clone(),
                seasons: Vec::new(),
            };
            shows.push(show);
            current_show = shows.len() - 1;
        }

        //determines whether the season exists in the seasons vector of the current show, if it does, it saves the index
        exists = false;
        let mut current_season: usize = 0;//content.show_season_episode.0.parse::<usize>().unwrap()
        for (i, season) in shows[current_show].seasons.iter().enumerate() {
            if season.number == content.show_season_episode.0.parse::<u8>().unwrap() {
                exists = true;
                current_season = i;
                break;
            }
        }

        //if the season doesn't exist in the current show's seasons vector, it creates it
        if !exists {
            let season = Season {
                number: content.show_season_episode.0.parse::<u8>().unwrap(),
                episodes: Vec::new()
            };

            shows[current_show].seasons.push(season);
            
            current_season = shows[current_show].seasons.len() - 1;
        }
        
        //push episode to current season
        shows[current_show].seasons[current_season].episodes.push(content);
    }



    if false {
        for show in &shows {
            //println!("{}", show.title);
            for season in &show.seasons {
                //println!("{}", season.number);
                for episode in &season.episodes {
                    println!("{}{}",
                        episode.parent_directory,
                        episode.original_filename,
                    );
                }
            }
        }
    }
    
    
    //unify generic and episode naming (bring together)

    //println!("Converting file to h265, no estimated time currently");
    //exec("ffmpeg -i W:/tlm/test_files/tf1.mp4 -c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k W:/tlm/test_files/tf1_h265.mp4");
}
