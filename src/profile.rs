use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;
use tracing::{error, debug};

use crate::pathbuf_to_string;

///Currently unused enum to allow filtering media by resolution standard
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum ResolutionStandard {
    UNKNOWN = 0,
    ED = 1,   //640
    SD = 2,   //720
    HD = 3,   //1280
    FHD = 4,  //1920
    WQHD = 5, //2560
    UHD = 6,  //3840/4096
}

impl ResolutionStandard {
    pub fn from(input: i32) -> Self {
        match input {
            1 => ResolutionStandard::SD,
            2 => ResolutionStandard::ED,
            3 => ResolutionStandard::HD,
            4 => ResolutionStandard::FHD,
            5 => ResolutionStandard::WQHD,
            6 => ResolutionStandard::UHD,
            _ => ResolutionStandard::UNKNOWN,
        }
    }

    pub fn from_wrapped(input: Option<i32>) -> Option<Self> {
        if let Some(input) = input {
            return Some(ResolutionStandard::from(input));
        }
        None
    }

    pub fn get_resolution_standard_from_width(width: i32) -> Self {
        match width {
            640 => ResolutionStandard::ED,
            720 => ResolutionStandard::SD,
            1280 => ResolutionStandard::HD,
            1920 => ResolutionStandard::FHD,
            2560 => ResolutionStandard::WQHD,
            3840 | 4096 => ResolutionStandard::UHD,
            _ => ResolutionStandard::UNKNOWN,
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            ResolutionStandard::ED => "ED".to_string(),
            ResolutionStandard::SD => "SD".to_string(),
            ResolutionStandard::HD => "HD".to_string(),
            ResolutionStandard::FHD => "FHD".to_string(),
            ResolutionStandard::WQHD => "WQHD".to_string(),
            ResolutionStandard::UHD => "UHD".to_string(),
            ResolutionStandard::UNKNOWN => panic!(),
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    SixteenByNine,
    TwentyOneByNine,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    //Chromecast:
    //  Gen 1 and 2:        H.264 High Profile up to level 4.1 (720p/60fps or 1080p/30fps)
    //  Gen 3 and Ultra:    H.264 High Profile up to level 4.2 (1080p/60fps)
    //  with Google TV:     H.264 High Profile up to level 5.1 (4Kx2K/30fps)
    H265,
    //Chromecast:
    //  Ultra:              HEVC / H.265 Main and Main10 Profiles up to level 5.1 (4K/60fps)
    //  with Google TV:     HEVC / H.265 Main and Main10 Profiles up to level 5.1 (4Kx2K@60fps)
    VP8,
    //Chromecast:
    //  Gen 1 and 2:        VP8 (720p/60fps or 1080p/30fps)
    //  Gen 3:              VP8 (720p/60fps or 1080p/30fps)
    //  Ultra:              VP8 (4K/30fps)
    VP9,
    //Chromecast:
    //  Ultra:              VP9 Profile 0 and Profile 2 up to level 5.1 (4K/60fps)
    //  with Google TV:     VP9 Profile-2 up to 4Kx2K@60fps
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum AudioCodec {
    FLAC, //(up to 96kHz/24-bit)
    HEAAC,
    LCAAC,
    MP3,
    Opus,
    Vorbis,
    WAVLPCM,
    WebM,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum Container {
    UNKNOWN = 0,
    MP4 = 1,
    MKV = 2,
    WEBM = 3,
}

impl Container {
    pub fn from(input: i32) -> Self {
        match input {
            0 => Container::MP4,
            1 => Container::MKV,
            2 => Container::WEBM,
            _ => Container::UNKNOWN,
        }
    }

    pub fn from_wrapped(input: Option<i32>) -> Option<Self> {
        if let Some(input) = input {
            return Some(Self::from(input));
        }
        None
    }

    pub fn get_container_from_extension(file_extension: String) -> Self {
        let file_extension = file_extension.to_lowercase(); //Hopefully this to_lowercase function is sufficient for the moment
        match file_extension.as_str() {
            "mp4" => Container::MP4,
            "mkv" => Container::MKV,
            "webm" => Container::WEBM,
            _ => Container::UNKNOWN,
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            Container::MP4 => "mp4".to_string(),
            Container::MKV => "mkv".to_string(),
            Container::WEBM => "webm".to_string(),
            Container::UNKNOWN => panic!(),
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Profile {
    pub width: Option<i32>,                              //Pixels
    pub height: Option<i32>,                             //Pixels
    pub framerate: Option<f64>,                          //FPS
    pub length_time: Option<f64>,                        //Seconds
    pub resolution_standard: Option<ResolutionStandard>, //Discounts the height difference, based on width
    //pub aspect_ratio: AspectRatio,                            //eg. SixteenByNine is 16:9
    pub container: Option<Container>, //Represents the file extension rather than specifically the container, as this may not be the case
                                      //TODO: Add current video information
                                      //TODO: Add current audio information
}

impl Profile {
    pub fn new(
        width: Option<i32>,
        height: Option<i32>,
        framerate: Option<f64>,
        length_time: Option<f64>,
        resolution_standard: Option<i32>,
        container: Option<i32>,
    ) -> Self {
        //resolution_standard
        let resolution_standard_i32: Option<i32> = resolution_standard;
        let mut resolution_standard: Option<ResolutionStandard> = None;
        if let Some(resolution_standard_i32) = resolution_standard_i32 {
            resolution_standard = Some(ResolutionStandard::from(resolution_standard_i32));
        }

        //container
        let container_i32: Option<i32> = container;
        let mut container: Option<Container> = None;
        if let Some(container_i32) = container_i32 {
            container = Some(Container::from(container_i32));
        }

        Self {
            width,
            height,
            framerate,
            length_time,
            resolution_standard,
            container,
        }
    }

    ///Create profile from a pathbuf
    pub fn from_file(full_path: &Path) -> Option<Profile> {
        let buffer;
        //linux & friends
        buffer = Command::new("mediainfo")
            .args(&["--output=JSON", &pathbuf_to_string(full_path)])
            .output()
            .unwrap_or_else(|err| {
                error!("Failed to execute process for mediainfo. Err: {}", err);
                panic!();
            });
        let temp= serde_json::from_str(from_utf8(&buffer.stdout).unwrap());
        if temp.is_ok() {
            let value: Value = temp.unwrap();
            let width = value["media"]["track"][1]["Width"]
            .to_string()
            .strip_prefix('"')?
            .strip_suffix('"')?
            .parse::<i32>()
            .unwrap();
            return Some(Self {
                width: Some(width),
                height: Some(
                    value["media"]["track"][1]["Height"]
                        .to_string()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .parse::<i32>()
                        .unwrap(),
                ),
                framerate: Some(
                    value["media"]["track"][1]["FrameRate"]
                        .to_string()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .parse::<f64>()
                        .unwrap(),
                ),
                length_time: Some(
                    value["media"]["track"][0]["Duration"]
                        .to_string()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .parse::<f64>()
                        .unwrap(),
                ),
                resolution_standard: Some(ResolutionStandard::get_resolution_standard_from_width(
                    width,
                )),
                container: Some(Container::get_container_from_extension(
                    value["media"]["track"][0]["FileExtension"]
                        .to_string()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .parse::<String>()
                        .unwrap(),
                )),
            });
        } else {
            return None;
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.width.is_none()
            || self.height.is_none()
            || self.framerate.is_none()
            || self.length_time.is_none()
            || self.resolution_standard.is_none()
            || self.container.is_none()
        {
            panic!("Tried to print empty profiles, don't really want to deal with this right now");
        }
        write!(
            f,
            "Width: {}, Height: {}, Framerate: {}, Length: {}, ResolutionStandard: {}, Container: {}",
            self.width.unwrap(), self.height.unwrap(), self.framerate.unwrap(), self.length_time.unwrap(), self.resolution_standard.unwrap() as i32, self.container.unwrap() as i32,
        )
    }
}

/* #[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct ConversionProfile {
    //Video
    basic_profile: BasicProfile,
    video_codec: Option<VideoCodec>,
    video_bitrate: Option<u32>,

    //Audio
    audio_codec: Option<AudioCodec>,
    audio_bitrate: Option<u32>,
    audio_samplerate: Option<u32>,
}

impl ConversionProfile {
    pub fn new(basic_profile: BasicProfile, full_path: PathBuf) -> Self {
        Self {
            basic_profile,
            video_codec: None,//TODO
            video_bitrate: None,//TODO
            audio_codec: None,//TODO
            audio_bitrate: None,//TODO
            audio_samplerate: None,//TODO
        }
    }
} */
