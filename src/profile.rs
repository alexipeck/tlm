use {
    crate::pathbuf_to_string,
    serde::{Deserialize, Serialize},
    serde_json::{Error, Value},
    std::{fmt, path::Path, process::Command, str::from_utf8},
    tracing::error,
};

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
}

impl fmt::Display for ResolutionStandard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ED => write!(f, "ED"),
            Self::SD => write!(f, "SD"),
            Self::HD => write!(f, "HD"),
            Self::FHD => write!(f, "FHD"),
            Self::WQHD => write!(f, "WQHD"),
            Self::UHD => write!(f, "UHD"),
            Self::UNKNOWN => write!(f, "UNKNOWN"),
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

    pub fn from_extension(file_extension: String) -> Self {
        let file_extension = file_extension.to_lowercase(); //Hopefully this to_lowercase function is sufficient for the moment
        match file_extension.as_str() {
            "mp4" => Container::MP4,
            "mkv" => Container::MKV,
            "webm" => Container::WEBM,
            _ => Container::UNKNOWN,
        }
    }
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MP4 => write!(f, "mp4"),
            Self::MKV => write!(f, "mkv"),
            Self::WEBM => write!(f, "webm"),
            Self::UNKNOWN => {
                error!("Container has unknown type.");
                panic!();
            }
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
        let temp: Result<Value, Error> = serde_json::from_str(from_utf8(&buffer.stdout).unwrap());
        if let Ok(value) = temp {
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
                container: Some(Container::from_extension(
                    value["media"]["track"][0]["FileExtension"]
                        .to_string()
                        .strip_prefix('"')?
                        .strip_suffix('"')?
                        .parse::<String>()
                        .unwrap(),
                )),
            });
        } else {
            None
        }
    }
}
