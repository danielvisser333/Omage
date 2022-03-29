pub mod engine;

use crossbeam_channel::{Receiver, Sender};
use slog::{crit, info, Logger};
use omage_util::PathManager;
use crate::engine::EngineThread;

pub struct Engine{
    logger : Logger,
    sender : Sender<EngineTask>,
    receiver : Receiver<EngineResult>,
}
impl Engine{
    pub fn new(app_name : &str) -> Self{
        let path_manager = PathManager::new(app_name);
        let logger = path_manager.create_logger();
        info!(logger, "Started the logger.");
        let (sender, thread_receiver) = crossbeam_channel::bounded(1);
        let (thread_sender, receiver) = crossbeam_channel::bounded(1);
        let thread_logger = logger.clone(); let thread_path_manager = path_manager.clone();
        rayon::spawn(|| {
            let engine = EngineThread::new(thread_sender, thread_receiver, thread_logger, thread_path_manager);
            engine.listen();
        });
        return Self{
            logger,sender,receiver,
        }
    }
    pub fn set_window_name(&self, name : String){
        self.sender.send(EngineTask::SetWindowName(name.clone())).unwrap();
        self.receiver.recv().unwrap();
        info!(self.logger, "Changed window title to {}.", name);
    }
    pub fn dispatch(self){
        if self.sender.send(EngineTask::Start).is_err(){crit!(self.logger, "Failed to start the engine")};
        while let Ok(result) = self.receiver.recv(){
            match result{
                EngineResult::Started => {info!(self.logger, "Successfully started the engine")}
                EngineResult::Finished => {break}
                _ => {}
            }
        }
        info!(self.logger, "Engine stopped.");
    }
}
#[derive(Clone, PartialEq)]
pub enum EngineTask{
    Start,
    SetWindowName(String),
}
#[derive(Copy, Clone, PartialEq)]
pub enum EngineResult{
    Finished,
    Started,
    Success,
}