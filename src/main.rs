//extern crate yaml_rust;
use regex::Regex;
use std::process::Command; //borrow::Cow,
use walkdir::WalkDir;

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
    //versions: &'g Vec<FileVersion>,
    //hash
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

fn main() {
    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    tracked_root_directories.push(String::from("/mnt/nas/tvshows/Breaking Bad/")); //manual entry
    tracked_root_directories.push(String::from("/mnt/nas/tvshows/Weeds/")); //manual entry

    //import all files in tracked root directories
    let mut raw_filepaths = Vec::new();
    for tracked_root_directory in tracked_root_directories {
        for entry in WalkDir::new(tracked_root_directory)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                raw_filepaths.push(entry.into_path());
            }
        }
    }

    //sort out filepaths into series and seasons
    let mut shows: Vec<Show> = Vec::new();
    
    for raw_filepath in raw_filepaths {
        let mut show_title = String::new();
        let original_filename = String::from(raw_filepath.file_name().unwrap().to_string_lossy());
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
        
        let season_episode_temp = re_strip(&original_filename, r"S[0-9]*E[0-9\-]*");

        let mut se_iter = season_episode_temp.split('E');
        let season_episode: (String, String) = (se_iter.next().unwrap().to_string(), se_iter.next().unwrap().to_string());

        let content = Content {
            parent_directory: String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/"),
            original_filename: original_filename,
            show_title: show_title,
            show_season_episode: season_episode,
            //show_season_episode: (season, episode), //manual entry
            //encoded_path: ,
            //path_depth: ,
            //versions: ,
        };

        //show_title
        let mut exists = false;
        for show in &shows {
            if show.title == content.show_title {
                exists = true;
                break;
            }
        }
        if !exists {
            let mut show = Show {
                title: content.show_title,
                seasons: Vec::new(),
            };
            
            let mut exists = false;
            for season in &show.seasons {
                if season.number == content.show_season_episode.0.parse::<u8>().unwrap() {
                    exists = true;
                    break;
                }
            }
            if !exists {
                let season = Season {
                    number: content.show_season_episode.1.parse::<u8>().unwrap(),
                    episodes: Vec::new()
                };

                show.seasons.push(season);
            }
        }
    }

    //unify generic and episode naming (bring together)
    for show in &shows {
        for season in &show.seasons {
            for episode in &season.episodes {
                println!("{}{}",
                    episode.parent_directory,
                    episode.original_filename,
                );
            }
        }
    }
    

    //parse out the title and store seperately

    //parse out the directory and store seperately

    //create simplified name of file, based on the title (only the title and episode id, no metadata)

    //change the name of (mv) the original file to the identifier
    //create a symlink of the file with it's original file name in another organised location, keeping the same folder structure it has before, ie, Castle->Season 1->Some Title.extension

    //println!("Converting file to h265, no estimated time currently");
    //exec("ffmpeg -i W:/tlm/test_files/tf1.mp4 -c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k W:/tlm/test_files/tf1_h265.mp4");
}
