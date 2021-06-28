//extern crate yaml_rust;
use std::{ops::Deref, os::raw, process::Command}; //borrow::Cow,
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
    //take in the path to every file in a directory
    //let command = r"find /mnt/nas/tvshows/ -name \*.\*";
    //let raw_structure = exec(command);

    let mut raw_paths = Vec::new();

    for entry in WalkDir::new("/mnt/nas/tvshows/")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            raw_paths.push(entry.into_path());
        }
    }
    for file in raw_paths {
        println!("{}", file.file_name().unwrap().to_string_lossy());
    }
    
    //iterate through each line
    /*let mut tracked_files: Vec<File> = Vec::new();
    for path in raw_structure.lines() {
        //iterate and store each / separated element

        //let mut split_path: Vec<&str> = Vec::new();

        //for full directory minus filename
        let mut parent_directory = String::new();
        let mut parent_directory_temp = String::new();
        let mut first = true;
        for path_sectioned in path.split('/') {
            parent_directory.push_str(&parent_directory_temp);
            if path_sectioned != "" {
                //for full directory minus filename
                if !first {
                    parent_directory_temp.push_str("/");
                } else {
                    first = false;
                }
                parent_directory_temp.push_str(path_sectioned);

                //split_path.insert(0, path_sectioned);
            }
        }
        //println!("{}", parent_directory);
        */
        let file = File {
            parent_directory: String::from(parent_directory + "/"),
            original_filename: String::from(parent_directory_temp),
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

    //Read list of files
    //let paths = fs::read_dir("./*/*").unwrap();

    /*for path in paths {
        println!("{}", path.unwrap().path().display())
    }*/

    /*
        let s =
    "
    foo:
        - list1
        - list2
    bar:
        - 1
        - 2.0
    ";
        let docs = YamlLoader::load_from_str(s).unwrap();

        // Multi document support, doc is a yaml::Yaml
        let doc = &docs[0];

        // Debug support
        println!("{:?}", doc);

        // Index access for map & array
        assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
        assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

        // Chained key/array access is checked and won't panic,
        // return BadValue if they are not exist.
        assert!(doc["INVALID_KEY"][100].is_badvalue());

        // Dump the YAML object
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(doc).unwrap(); // dump the YAML object to a String
        }
        println!("{}", out_str);
        */
}
