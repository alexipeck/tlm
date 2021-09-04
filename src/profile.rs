use serde_json::Value;
use std::fmt;
use std::process::Command;
use std::str::from_utf8;

use std::path::PathBuf;

pub enum ResolutionStandard {
    UHD,
    WQHD,
    FHD,
    HD,
    ED,
    SD,
}
#[allow(dead_code)]
pub struct EncodingProfile {
    _resolution_standard: ResolutionStandard,
}

#[derive(Clone, Debug, Copy)]
pub struct Profile {
    pub width: u32,
    pub height: u32,
    pub framerate: f64,
    pub length_time: f64,
}

impl Profile {
    pub fn new(width: u32, height: u32, framerate: f64, length_time: f64) -> Self {
        Profile {
            width,
            height,
            framerate,
            length_time,
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Width: {}, Height: {}, Framerate: {}, Length: {}",
            self.width, self.height, self.framerate, self.length_time
        )
    }
}

impl Profile {
    pub fn from_file(path: PathBuf) -> Option<Self> {
        let buffer;
        //linux & friends
        buffer = Command::new("mediainfo")
            .args(&["--output=JSON", path.to_str().unwrap()])
            .output()
            .expect("failed to execute process");

        let v: Value = serde_json::from_str(from_utf8(&buffer.stdout).unwrap()).unwrap();

        Some(Profile {
            width: v["media"]["track"][1]["Width"]
                .to_string()
                .strip_prefix('"')?
                .strip_suffix('"')?
                .parse::<u32>()
                .unwrap(),
            height: v["media"]["track"][1]["Height"]
                .to_string()
                .strip_prefix('"')?
                .strip_suffix('"')?
                .parse::<u32>()
                .unwrap(),
            framerate: v["media"]["track"][1]["FrameRate"]
                .to_string()
                .strip_prefix('"')?
                .strip_suffix('"')?
                .parse::<f64>()
                .unwrap(),
            length_time: v["media"]["track"][0]["Duration"]
                .to_string()
                .strip_prefix('"')?
                .strip_suffix('"')?
                .parse::<f64>()
                .unwrap(),
        })
    }
}
