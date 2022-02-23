use std::ffi::CStr;
use std::fs::OpenOptions;
use std::os::raw::c_char;
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::{Drain, error, info, Logger, o, warn};

#[derive(Clone)]
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
    pub fn get_log_path(&self) -> PathBuf{
        let time = chrono::Utc::now();
        return self.log_directory.join(format!("./{}.log",time.format("%Y-%m-%d-%H-%M-%S")));
    }
    pub fn create_logger(&self) -> Logger{
        let term_decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(term_decorator).build().fuse();
        let drain_async = slog_async::Async::new(drain).build().fuse();
        let log_path = self.get_log_path();
        if !log_path.parent().unwrap().exists(){std::fs::create_dir_all(log_path.parent().unwrap()).expect("Failed to create log directory")};
        let log_file = OpenOptions::new().create(true).write(true).truncate(true).open(log_path).expect("Failed to create log file");
        let file_decorator = slog_term::PlainSyncDecorator::new(log_file);
        let file_drain = slog_term::FullFormat::new(file_decorator).build().fuse();
        let file_drain_async = slog_async::Async::new(file_drain).build().fuse();
        let double_drain = slog::Duplicate::new(drain_async,file_drain_async).fuse();
        let logger = slog::Logger::root(double_drain, o!());
        info!(logger, "Initialized logger");
        return logger;
    }
    pub fn load<T>(&self, name : &str, file_type : FileType, logger : &Logger) -> Option<T>
    where T : DeserializeOwned{
        let path = self.get_path(file_type, name);
        return if !path.exists() { None } else {
            match std::fs::read(path.clone()) {
                Ok(config) => {
                    match toml::from_slice(&config) {
                        Ok(config) => { Some(config) }
                        Err(error) => {
                            warn!(logger, "Failed to parse config, {}",error);
                            if let Err(error) = std::fs::remove_file(path.clone()){
                                warn!(logger, "Failed to reset config, {}, {}",path.to_string_lossy(), error);
                            };
                            None
                        }
                    }
                }
                Err(error) => {
                    warn!(logger, "Failed to load file, {}, {}", path.to_string_lossy(), error);
                    None
                }
            }
        }
    }
    pub fn load_or_default<T>(&self, name : &str, file_type : FileType, logger : &Logger) -> T
    where T : DeserializeOwned + Default{
        return match self.load(name, file_type, logger){
            Some(config) => {config}
            None => {info!(logger, "File does not exist, {}, {:?}", name, file_type);T::default()}
        }
    }
    fn get_path(&self, file_type : FileType, name : &str) -> PathBuf{
        return match file_type{
            FileType::Config => {self.config_directory.join(format!("./{}.toml",name))}
            FileType::Cache => {self.cache_directory.join(format!("./{}.cache",name))}
        };
    }
    pub fn save<T>(&self, name : &str, file_type : FileType, logger : &Logger, config : &T)
    where T : Serialize{
        let path = self.get_path(file_type, name);
        if !path.parent().unwrap().exists(){match std::fs::create_dir_all(path.parent().unwrap()){
            Ok(_)=>{}
            Err(error) => {
                error!(logger, "Failed to create config directory, {}, {}",path.to_string_lossy(), error);
                return;
            }
        }}
    match toml::to_vec(config){
        Ok(config) => {
            match std::fs::write(path.clone(), config){
                Ok(_) => {return}
                Err(error) => {
                    warn!(logger, "Failed to save config {}, {}", path.to_string_lossy(), error);
                }
            }
        }
        Err(error) => {
            error!(logger, "Failed to serialize {} config, {}", name, error);
        }
    }
    }
}
#[derive(Copy, Clone, Debug)]
pub enum FileType{
    Config,
    Cache,
}
pub unsafe fn get_string_from_slice(slice : &[c_char]) -> String{
    return CStr::from_ptr(slice.as_ptr()).to_str().unwrap().to_owned();
}