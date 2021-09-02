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
    resolution_standard: ResolutionStandard,
}

#[derive(Clone, Debug)]
pub struct Profile {
    width: usize,
    height: usize,
    framerate: usize,
    length_time: usize,
}

impl Profile {
    pub fn new(width: usize, height: usize, framerate: usize, length_time: usize) -> Self {
        Profile {
            width,
            height,
            framerate,
            length_time,
        }
    }
}
