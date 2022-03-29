use crossbeam_channel::{Receiver, Sender};
use slog::{info, Logger};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(target_os = "linux")]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopExtWindows;
use winit::window::Window;
use omage_renderer::instance::RenderInstance;
use omage_renderer::Renderer;
use omage_util::{FileType, PathManager};
use crate::{EngineResult, EngineTask};

pub struct EngineThread{
    receiver : Receiver<EngineTask>,
    sender : Sender<EngineResult>,
    logger : Logger,
    event_loop : EventLoop<()>,
    window : Window,
    renderer : Renderer,
}
impl EngineThread{
    pub fn new(sender : Sender<EngineResult>, receiver : Receiver<EngineTask>, logger : Logger, path_manager : PathManager) -> Self{
        info!(logger, "[thread#{}]Creating new engine.", rayon::current_thread_index().unwrap());
        let event_loop = EventLoop::new_any_thread();
        let window = Window::new(&event_loop).unwrap();
        let render_config = path_manager.load_file_or_default(&logger, "render", FileType::Config);
        let render_instance = unsafe{RenderInstance::new(logger.clone(), &window, render_config)};
        let renderer = Renderer::new(render_instance, path_manager);
        return Self{
            sender,receiver,logger,event_loop,window,renderer,
        }
    }
    pub fn listen(self){
        while let Ok(task) = self.receiver.recv(){
            match task{
                EngineTask::Start => {break;}
                EngineTask::SetWindowName(name) => {self.window.set_title(&name); self.sender.send(EngineResult::Success).unwrap()}
            }
        }
        self.start();
    }
    fn start(mut self){
        info!(self.logger, "[thread#{}]Starting the event loop", rayon::current_thread_index().unwrap());
        self.event_loop.run_return(|event, _, control_flow|{
            match event{
                Event::WindowEvent {window_id : _, event} => {
                    match event{
                        WindowEvent::CloseRequested => {*control_flow = ControlFlow::Exit}
                        _ => {}
                    }
                }
                _ => {}
            }
        });
        info!(self.logger, "[thread#{}]Engine stopping.", rayon::current_thread_index().unwrap());
        drop(self.logger);
        self.renderer.stop();
        self.sender.send(EngineResult::Finished).unwrap();
    }
}