use serde_json::Value;
use std::fmt;
use std::process::Command;
use std::str::from_utf8;
use tracing::error;

use std::path::PathBuf;

///Currently unused enum to allow filtering media by resolution standard
pub enum ResolutionStandard {
    UHD,
    WQHD,
    FHD,
    HD,
    ED,
    SD,
}

///Currently unused encoding profile used to set ffmpeg flags based on resolution
///standard
#[allow(dead_code)]
pub struct EncodingProfile {
    _resolution_standard: ResolutionStandard,
}

///Struct to store media information collected from media info
///which will then be used to filter media and to set ffmpeg flags
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
    ///Create profile from a pathbuf
    pub fn from_file(path: PathBuf) -> Option<Self> {
        let buffer;
        //linux & friends
        buffer = Command::new("mediainfo")
            .args(&["--output=JSON", path.to_str().unwrap()])
            .output()
            .unwrap_or_else(|err| {
                error!("Failed to execute process for mediainfo. Err: {}", err);
                panic!();
            });

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
