use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::process::Command;
use std::str::from_utf8;
use tracing::error;

use std::path::PathBuf;

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
    pub fn get_resolution_standard_from_width(width: u32) -> Self {
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

pub fn convert_i32_to_resolution_standard(input: i32) -> ResolutionStandard {
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

/* impl ToSql<Text, Pg> for ResolutionStandard
where
    Pg: Backend,
    String: ToSql<Text, Pg>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            ResolutionStandard::SD => String::from("SD").to_sql(out),
            ResolutionStandard::ED => String::from("ED").to_sql(out),
            ResolutionStandard::HD => String::from("HD").to_sql(out),
            ResolutionStandard::FHD => String::from("FHD").to_sql(out),
            ResolutionStandard::WQHD => String::from("WQHD").to_sql(out),
            ResolutionStandard::UHD => String::from("UHD").to_sql(out),
        }
    }
} */

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

pub fn convert_i32_to_container(input: i32) -> Container {
    match input {
        0 => Container::MP4,
        1 => Container::MKV,
        2 => Container::WEBM,
        _ => Container::UNKNOWN,
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct BasicProfile {
    pub width: u32,                              //Pixels
    pub height: u32,                             //Pixels
    pub framerate: f64,                          //FPS
    pub length_time: f64,                        //Seconds
    pub resolution_standard: ResolutionStandard, //Discounts the height difference, based on width
    //pub aspect_ratio: AspectRatio,                //eg. SixteenByNine is 16:9
    pub container: Container, //Represents the file extension rather than specifically the container, as this may not be the case
                              //TODO: Add current video information
                              //TODO: Add current audio information
}

impl fmt::Display for BasicProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Width: {}, Height: {}, Framerate: {}, Length: {}, ResolutionStandard: {}, Container: {}",
            self.width, self.height, self.framerate, self.length_time, self.resolution_standard as i32, self.container as i32,
        )
    }
}

impl BasicProfile {
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
        let width = v["media"]["track"][1]["Width"]
            .to_string()
            .strip_prefix('"')?
            .strip_suffix('"')?
            .parse::<u32>()
            .unwrap();

        Some(Self {
            width,
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
            resolution_standard: ResolutionStandard::get_resolution_standard_from_width(width),
            container: Container::get_container_from_extension(
                v["media"]["track"][0]["FileExtension"]
                    .to_string()
                    .strip_prefix('"')?
                    .strip_suffix('"')?
                    .parse::<String>()
                    .unwrap(),
            ),
        })
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct ConversionProfile {
    //Video
    basic_profile: BasicProfile,
    video_codec: Option<VideoCodec>,
    video_birate: Option<u32>,

    //Audio
    audio_codec: Option<AudioCodec>,
    audio_bitrate: Option<u32>,
    audio_samplerate: Option<u32>,
}

///Struct to store media information collected from media info
///which will then be used to filter media and to set ffmpeg flags
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Profile {
    //Current
    pub current_profile: BasicProfile,

    //Future
    pub future_profile: ConversionProfile,
}

impl Profile {
    /* pub fn new() -> Self {
        Self {

        }
    } */
}
