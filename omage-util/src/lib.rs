use std::path::PathBuf;
use slog::Logger;

pub struct PathManager{
    pub config_directory : PathBuf,
    pub cache_directory : PathBuf,
    pub log_directory : PathBuf,
}
impl PathManager{
    pub fn from_app_name(app_name : &str) -> Self{
        let paths = directories::ProjectDirs::from("com", "omage", app_name).unwrap();
        let config_directory = paths.config_dir().to_path_buf();
        let cache_directory = paths.cache_dir().to_path_buf();
        let log_directory = config_directory.join("./logs");
        return Self{
            config_directory,
            cache_directory,
            log_directory,
        }
    }
}