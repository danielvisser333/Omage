use crossbeam_channel::{Receiver, Sender};
use slog::{Drain, error, info, Logger};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(target_os = "linux")]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopExtWindows;
use winit::window::Window;
use omage_renderer::Renderer;
use omage_util::PathManager;

pub struct Engine{
    logger : Logger,
    path_manager : PathManager,
    sender : Sender<EngineTask>,
    receiver : Receiver<EngineResult>,
}
impl Engine{
    pub fn new(app_name : &str) -> Self {
        let path_manager = omage_util::PathManager::from_app_name(app_name);
        let logger = path_manager.create_logger();
        let logger_thread = logger.clone(); let path_manager_thread = path_manager.clone();
        let (task_sender, task_receiver) = crossbeam_channel::bounded(2);
        let (result_sender, result_receiver) = crossbeam_channel::bounded(2);
        rayon::spawn(move ||{
            let logger = logger_thread;
            info!(logger, "Started the event thread");
            let mut event_loop = EventLoop::new_any_thread();
            let mut engine = EngineBackend::new(logger , path_manager_thread , &event_loop , task_receiver, result_sender);
            event_loop.run_return(|event, _, control_flow|{
                engine.event(event,control_flow);
            });
            engine.stop();
        });
        info!(logger, "Successfully initialized engine");
        return Self {
            path_manager,
            logger,
            sender : task_sender,
            receiver : result_receiver,
        }
    }
    pub fn await_close_request(self){
        while self.receiver.recv().unwrap() != EngineResult::Closed{}
        drop(self.logger);
    }
}
struct EngineBackend{
    logger : Logger,
    main_window : Window,
    receiver : Receiver<EngineTask>,
    sender : Sender<EngineResult>,
    renderer : Renderer,
}
impl EngineBackend{
    pub fn new(logger : Logger, path_manager : PathManager , event_loop : &EventLoop<()>, receiver : Receiver<EngineTask> , sender : Sender<EngineResult>,) -> Self{
        let main_window = match Window::new(event_loop){
            Ok(window) => {info!(logger, "Created main window");window}
            Err(error) => {error!(logger, "Failed to create window, {}", error); panic!()}
        };
        let window_size = main_window.inner_size();
        let renderer = Renderer::new(logger.clone(), path_manager, &main_window);
        return Self{
            logger,sender,receiver,main_window,renderer
        }
    }
    pub fn event(&mut self, event : Event<()>, control_flow : &mut ControlFlow){
        match event{
            Event::WindowEvent {event, window_id} => {
                match event{
                    WindowEvent::CloseRequested => {
                        if window_id == self.main_window.id(){
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _=>{}
                }
            }
            _=>{}
        }
        self.tick();
    }
    pub fn tick(&mut self){

    }
    pub fn stop(self){
        info!(self.logger, "Close button pressed, stopping");
        self.renderer.stop();
        drop(self.logger);
        self.sender.send(EngineResult::Closed).unwrap();
    }
}
enum EngineTask{

}
#[derive(PartialEq)]
enum EngineResult{
    Closed,
}