mod functions;

use ash::{Device, Entry, Instance};
use ash::extensions::khr::Surface;
use ash::vk::SurfaceKHR;
use crossbeam_channel::{Receiver, Sender};
use slog::{error, info, Logger};
use winit::window::Window;
use serde_derive::{Serialize,Deserialize};
use omage_util::{FileType, PathManager};
use crate::functions::device::QueueInfo;

pub struct Renderer{
    logger : Logger,
    path_manager : PathManager,
    sender : Sender<RenderTask>,
    receiver : Receiver<RenderResult>,
}
impl Renderer{
    pub fn new(logger : Logger, path_manager : PathManager, window : &Window) -> Self{
        let (task_sender, task_receiver) = crossbeam_channel::bounded(2);
        let (result_sender, result_receiver) = crossbeam_channel::bounded(2);
        let render_config = path_manager.load_or_default("renderer", FileType::Config, &logger);
        let renderer = unsafe { RenderInstance::new(&logger, &render_config, window) };
        rayon::spawn(move ||{
            let renderer = RenderBackend::new(renderer);
            let mut task = task_receiver.try_recv();
            while task != Ok(RenderTask::Stop){
                renderer.draw();
                task = task_receiver.try_recv();
            }
            drop(renderer);
            result_sender.send(RenderResult::Success).unwrap();
        });
        return Self{
            logger,
            path_manager,
            sender : task_sender,
            receiver : result_receiver,
        };
    }
    pub fn stop(self){
        self.sender.send(RenderTask::Stop).unwrap();
        while let Ok(message) = self.receiver.recv(){
            if message == RenderResult::Success{break};
        }
        drop(self.logger);
    }
}
#[derive(PartialEq)]
enum RenderTask{
    Stop
}
#[derive(PartialEq)]
enum RenderResult{
    Success
}
struct RenderInstance{
    entry : Entry,
    instance : Instance,
    surface : SurfaceKHR,
    logger : Logger,
}
impl RenderInstance{
    pub unsafe fn new(logger : &Logger, config : &RenderConfig, window : &Window) -> Self{
        let entry = match Entry::load(){
            Ok(entry) => {
                info!(logger, "Successfully loaded Vulkan driver");
                entry
            }
            Err(error) => {
                error!(logger, "Failed to load Vulkan driver, {}", error);
                panic!();
            }
        };
        let instance = functions::instance::create_instance(logger, &entry, config, config.validation, window);
        let surface = match ash_window::create_surface(&entry, &instance, window, None){
            Ok(surface) => {
                info!(logger, "Successfully created Vulkan surface");
                surface
            }
            Err(error) => {
                error!(logger, "Failed to create Vulkan surface, {}", error);
                panic!();
            }
        };
        return Self{
            entry,instance,surface,logger:logger.clone(),
        }
    }
}
#[derive(Serialize,Deserialize)]
pub struct RenderConfig{
    validation : bool,
    app_name : String,
    engine_name : String,
    app_version : u32,
    engine_version : u32,
}
impl Default for RenderConfig{
    fn default() -> Self {
        return Self{
            validation : true,
            app_version : 1,
            engine_version : 1,
            app_name : String::from("omage-app"),
            engine_name : String::from("omage")
        }
    }
}
struct RenderBackend{
    logger : Logger,
    _entry : Entry,
    instance : Instance,
    surface_loader : Surface,
    surface : SurfaceKHR,
    device : Device,
}
impl RenderBackend{
    pub fn new(instance : RenderInstance) -> Self{
        let logger = instance.logger;
        let entry = instance.entry;
        let surface = instance.surface;
        let instance = instance.instance;
        let surface_loader = Surface::new(&entry, &instance);
        let physical_device = unsafe{functions::device::select_physical_device(&logger, &instance, &surface_loader, surface)};
        let queue_info = unsafe{QueueInfo::new(&instance, physical_device, &surface_loader, surface, &logger)};
        let device = unsafe{functions::device::create_device(&logger, &instance, physical_device, &queue_info)};
        info!(logger, "Renderer creation completed");
        return Self{
            _entry:entry,instance,surface_loader,surface,logger,device,
        }
    }
    pub fn draw(&self){

    }
}
impl Drop for RenderBackend{
    fn drop(&mut self) {
        unsafe {
            match self.device.device_wait_idle(){
                Ok(_) => {
                    info!(self.logger, "Successfully waited for render operations to complete");
                }
                Err(error) => {
                    error!(self.logger, "Failed to shutdown renderer, {}", error);
                }
            };

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
            info!(self.logger, "Destroyed renderer");
        }
    }
}