//extern crate yaml_rust;
use std::{process::Command}; //borrow::Cow,
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

struct File {
    parent_directory: String,
    original_filename: String,
    //encoded_filename: &'d str,
    //encoded_path: &'e str,
    //path_depth: &'f u8,
    //versions: &'g Vec<FileVersion>,
    //hash
}

fn main() {
    //let command = r"find /mnt/nas/tvshows/ -name \*.\*";
    //let raw_structure = exec(command);

    //tracked directories - avoid crossover, it will lead to duplicate entries
    let mut tracked_root_directories: Vec<String> = Vec::new();
    tracked_root_directories.push(String::from("/mnt/nas/tvshows/Breaking Bad/"));
    tracked_root_directories.push(String::from("/mnt/nas/tvshows/Weeds/"));

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
    
    let mut tracked_files: Vec<File> = Vec::new();
    for raw_filepath in raw_filepaths {
        let file = File {
            parent_directory: String::from(raw_filepath.parent().unwrap().to_string_lossy() + "/"),
            original_filename: String::from(raw_filepath.file_name().unwrap().to_string_lossy()),
            //encoded_filename: ,
            //encoded_path: ,
            //path_depth: ,
            //versions: ,
        };

        //Saving
        tracked_files.push(file);
    }

    for file in tracked_files {
        println!(
            "Parent: {} Filename: {}",
            file.parent_directory, file.original_filename
        );
    }

    //parse out the title and store seperately

    //parse out the directory and store seperately

    //create simplified name of file, based on the title (only the title and episode id, no metadata)

    //change the name of (mv) the original file to the identifier
    //create a symlink of the file with it's original file name in another organised location, keeping the same folder structure it has before, ie, Castle->Season 1->Some Title.extension

    //println!("Converting file to h265, no estimated time currently");
    //exec("ffmpeg -i W:/tlm/test_files/tf1.mp4 -c:v libx265 -crf 25 -preset slower -profile:v main -c:a aac -q:a 224k W:/tlm/test_files/tf1_h265.mp4");

}
