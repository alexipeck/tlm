#[cfg(test)]
mod tests {
    use crate::config::ServerConfig;

    //Tests the generation and activation of an Encode, will somehow test a remote worker encoding it,
    //but it shouldn't be that hard, it will just involve creating a dummy instance of the entire program :)
    #[test]
    fn test_encode() {
        let file_version_model: crate::model::FileVersionModel = crate::model::FileVersionModel {
            id: 0,
            generic_uid: 0,
            full_path: r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv".to_string(),
            master_file: true,
            file_hash: None,
            fast_file_hash: None,
            width: None,
            height: None,
            framerate: None,
            length_time: None,
            resolution_standard: None,
            container: None,

        };
        let file_version: crate::generic::FileVersion = crate::generic::FileVersion::from_file_version_model(file_version_model);
        let encode_profile: crate::encode::EncodeProfile = crate::encode::EncodeProfile::H265_TV_1080p;
        let server_config: std::sync::Arc<std::sync::RwLock<crate::config::ServerConfig>> = std::sync::Arc::new(std::sync::RwLock::new(ServerConfig::default()));
        let mut encode: crate::encode::Encode = crate::encode::Encode::new(&file_version, &encode_profile, &server_config);
        encode.encode_string.activate(std::env::temp_dir());
    }
    
    #[test]
    fn test_get_show_title_from_pathbuf() {
        assert_eq!(
            crate::get_show_title_from_pathbuf(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz".to_string()
        );
        assert_eq!(
            crate::get_show_title_from_pathbuf(&std::path::PathBuf::from(
                r"T:\Alcatraz\Season 1\Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz".to_string()
        );
        assert_eq!(
            crate::get_show_title_from_pathbuf(&std::path::PathBuf::from(
                r"\\192.168.2.30\tvshows\Alcatraz\Season 1\Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz".to_string()
        );
        assert_eq!(
            crate::get_show_title_from_pathbuf(&std::path::PathBuf::from(
                r"/media/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz".to_string()
        );
    }

    //Lib.rs
    #[test]
    fn test_os_string_to_string() {
        assert_eq!(
            crate::os_string_to_string(&std::ffi::OsString::from(
                r"T:\Alcatraz\Season 1\Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"T:\Alcatraz\Season 1\Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv".to_string()
        );
    }

    #[test]
    fn test_pathbuf_to_string() {
        assert_eq!(
            crate::pathbuf_to_string(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
                .to_string()
        );
    }

    #[test]
    fn test_pathbuf_with_suffix() {
        assert_eq!(
            crate::pathbuf_with_suffix(
                &std::path::PathBuf::from(
                    r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
                ),
                "_test".to_string()
            ),
            std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p_test.mkv"
            )
        );
    }

    #[test]
    fn test_get_file_stem() {
        assert_eq!(
            crate::get_file_stem(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz - S01E02 - Ernest Cobb HDTV-720p".to_string()
        );
    }

    #[test]
    fn test_get_file_name() {
        assert_eq!(
            crate::get_file_name(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            r"Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv".to_string()
        );
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(
            crate::get_extension(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            "mkv".to_string()
        )
    }

    #[test]
    fn test_get_parent_directory() {
        assert_eq!(
            crate::get_parent_directory(&std::path::PathBuf::from(
                r"/mnt/tvshows/Alcatraz/Season 1/Alcatraz - S01E02 - Ernest Cobb HDTV-720p.mkv"
            )),
            std::path::PathBuf::from(r"/mnt/tvshows/Alcatraz/Season 1/")
        )
    }
}