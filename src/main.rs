use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command; //borrow::Cow, thread::current,
use std::time::Instant;
use twox_hash::xxh3;
use walkdir::WalkDir;
use tlm::{Content, Designation, Season, Show, Shows, Queue, re_strip, prioritise_content_by_title, prioritise_content_by_uid};

fn hash_file(path: PathBuf) -> u64 {
    println!("Hashing: {}...", path.display());
    let timer = Instant::now();
    let hash = xxh3::hash64(&fs::read(path.to_str().unwrap()).unwrap());
    println!("Took: {}ms", timer.elapsed().as_millis());
    println!("Hash was: {}", hash);
    hash
}



fn rename(source: &String, target: &String) {
    let rename_string: Vec<&str> = vec!["-f", &source, &target];

    if !cfg!(target_os = "windows") {
        //linux & friends
        Command::new("mv")
            .args(rename_string)
            .output()
            .expect("failed to execute process");
    } else {
        //windows
        /* Command::new("mv")
            .args(rename_string)
            .output()
            .expect("failed to execute process"); */
    }
}
//needs to handle the target filepath already existing, overwrite
fn encode(source: &String, target: &String) -> String { //command: &Vec<&str>
    let encode_string: Vec<&str> = vec!["-i", &source, "-c:v", "libx265", "-crf", "25", "-preset", "slower", "-profile:v", "main", "-c:a", "aac", "-q:a", "224k", &target];
    
    let buffer;
    if !cfg!(target_os = "windows") {
        //linux & friends
        buffer = Command::new("ffmpeg")
            .args(encode_string)
            .output()
            .expect("failed to execute process");
    } else {
        //windows
        buffer = Command::new("ffmpeg")
            .args(encode_string)
            .output()
            .expect("failed to execute process");
    }
    String::from_utf8_lossy(&buffer.stdout).to_string()
}





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
    //Queue
    let mut queue = Queue {
        priority_queue: Vec::new(),
        main_queue: Vec::new(),
    };

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    if !cfg!(target_os = "windows") {
        //tracked_root_directories.push(String::from("/mnt/nas/tvshows")); //manual entry
        tracked_root_directories.push(String::from("/home/anpeck/tlm/test_files")); //manual entry
    } else {
        //tracked_root_directories.push(String::from("T:/")); //manual entry
        tracked_root_directories.push(String::from(r"C:\Users\Alexi Peck\Desktop\tlm\test_files\generic\"));
        tracked_root_directories.push(String::from(r"C:\Users\Alexi Peck\Desktop\tlm\test_files\episode\"));
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
    let mut shows = Shows::new();

    //loop through all paths
    for raw_filepath in raw_filepaths {
        let mut content = Content::new(&raw_filepath);
        if content.show_title.is_some() {
            content.set_show_uid(shows.ensure_show_exists_by_title(content.show_title.clone().unwrap()).0);
        }

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
                let season_episode = content.show_season_episode;
                content.show_title = Some(show_title);
                content.show_season_episode = season_episode;

                //saves index of the current show in the shows vector
                //ensures show exists, saving the index and uid

                //push episode
                shows.add_episode(content.clone());
            }
            /*Designation::Movie => (

            ),*/
            _ => {}
        }
        queue.main_queue.push(content);
    }
    let filenames: Vec<String> = Vec::new();
    //filenames.push(String::from(r"Weeds - S08E10 - Threshold Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E11 - God Willing and the Creek Don't Rise Bluray-1080p.mkv"));
    //filenames.push(String::from(r"Weeds - S08E12-13 - It's Time Bluray-1080p.mkv"));

    let uids: Vec<usize> = Vec::new();
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
        let encode_target = format!("{}{}_encode.mp4", content.parent_directory, content.filename_woe);
        let rename_target = format!("{}{}.mp4", content.parent_directory, content.filename_woe);
        println!("Starting encode of {}\nEncoding to {}_encode.mp4", content.filename, content.filename_woe);
        encode(&source, &encode_target);
        rename(&encode_target, &rename_target);
    }

    //shows.print();
    //add to db by filename, allowing the same file to be retargeted in another directory, without losing track of all the data associated with the episode

    //unify generic and episode naming (bring together)
}
