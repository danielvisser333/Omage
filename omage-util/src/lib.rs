use std::fs::OpenOptions;
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use serde::Serialize;
use slog::{Drain, Duplicate, Logger, o, warn};
use slog_async::Async;
use slog_term::{FullFormat, PlainSyncDecorator, TermDecorator};

#[derive(Clone)]
pub struct PathManager{
    config_directory : PathBuf,
    cache_directory : PathBuf,
}
impl PathManager{
    pub fn new(app_name : &str) -> Self{
        let project_dirs = directories::ProjectDirs::from("com", "omage", app_name).unwrap();
        return Self{
            config_directory : project_dirs.config_dir().to_path_buf(),
            cache_directory : project_dirs.cache_dir().to_path_buf(),
        }
    }
    pub fn create_logger(&self) -> Logger{
        let term_decorator = TermDecorator::new().build();
        let term_drain = FullFormat::new(term_decorator).build().fuse();
        let async_term_drain = Async::new(term_drain).build().fuse();
        let log_file_name = chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S");
        let log_file_path = self.config_directory.join(format!("./logs/{}.log", log_file_name));
        if !log_file_path.parent().unwrap().exists(){std::fs::create_dir_all(log_file_path.parent().unwrap()).expect("Failed to create log directory");}
        let log_file = OpenOptions::new().create(true).write(true).truncate(true).open(log_file_path).unwrap();
        let file_decorator = PlainSyncDecorator::new(log_file);
        let file_drain = FullFormat::new(file_decorator).build().fuse();
        let async_file_drain = Async::new(file_drain).build().fuse();
        let full_drain = Duplicate::new(async_term_drain, async_file_drain).fuse();
        return Logger::root(full_drain, o!());
    }
    fn get_path(&self, name : &str, file_type : FileType) -> PathBuf{
        return match file_type{
            FileType::Config => {self.config_directory.join(format!("./{}.toml",name))}
            FileType::Cache => {self.cache_directory.join(format!("./{}.cache", name))}
        }
    }
    pub fn load_file<T>(&self, logger : &Logger, name : &str, file_type : FileType) -> Option<T>
    where T : DeserializeOwned{
        let path = self.get_path(name, file_type);
        return if !path.exists() { None } else {
            let file_data = std::fs::read(path.clone()).unwrap();
            match toml::from_slice(&file_data) {
                Ok(config) => { Some(config) }
                Err(error) => {
                    warn!(logger, "Invalid config {:?}, {}", path, error);
                    None
                }
            }
        }
    }
    pub fn load_file_or_default<T>(&self, logger : &Logger, name : &str, file_type : FileType) -> T
    where T : Serialize + DeserializeOwned + Default{
        return match self.load_file(logger, name, file_type){
            Some(file) => {file}
            None => {
                let data = T::default();
                self.save_file(name, file_type, &data);
                data
            }
        }
    }
    pub fn save_file<T>(&self, name : &str, file_type : FileType, file : &T)
    where T : Serialize{
        let path = self.get_path(name, file_type);
        if !path.parent().unwrap().exists(){std::fs::create_dir_all(path.parent().unwrap()).unwrap()}
        let data = toml::to_vec(file).unwrap();
        std::fs::write(path, data).unwrap();
    }
}
#[derive(Copy, Clone)]
pub enum FileType{
    Config,
    Cache,
}